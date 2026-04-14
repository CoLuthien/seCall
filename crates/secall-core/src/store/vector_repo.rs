use crate::search::vector::VectorRow;
use crate::store::db::Database;

pub trait VectorRepo {
    fn init_vector_table(&self) -> anyhow::Result<()>;
    fn insert_vector(
        &self,
        embedding: &[f32],
        session_id: &str,
        turn_index: u32,
        chunk_seq: u32,
        model: &str,
    ) -> anyhow::Result<i64>;
    fn search_vectors(
        &self,
        query_embedding: &[f32],
        limit: usize,
        session_ids: Option<&[String]>,
    ) -> crate::error::Result<Vec<VectorRow>>;
    /// rowid로 turn_vectors의 (session_id, turn_index, chunk_seq) 조회.
    /// ANN 검색 결과를 DB 메타데이터와 연결할 때 사용.
    fn get_vector_meta(&self, rowid: i64) -> anyhow::Result<(String, u32, u32)>;
}

// VectorRepo impl for Database — vector table management + search
impl VectorRepo for Database {
    fn init_vector_table(&self) -> anyhow::Result<()> {
        self.conn().execute_batch(
            "
            CREATE TABLE IF NOT EXISTS turn_vectors (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id  TEXT NOT NULL,
                turn_index  INTEGER NOT NULL,
                chunk_seq   INTEGER NOT NULL,
                model       TEXT NOT NULL,
                embedded_at TEXT NOT NULL,
                embedding   BLOB NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_vectors_session ON turn_vectors(session_id);
        ",
        )?;
        Ok(())
    }

    fn insert_vector(
        &self,
        embedding: &[f32],
        session_id: &str,
        turn_index: u32,
        chunk_seq: u32,
        model: &str,
    ) -> anyhow::Result<i64> {
        if embedding.is_empty() {
            anyhow::bail!("empty embedding for session={session_id} turn={turn_index}");
        }

        // 기존 데이터와 차원 일치 확인 (첫 삽입 시 건너뜀)
        let existing_dim: Option<usize> = self
            .conn()
            .query_row(
                "SELECT LENGTH(embedding) FROM turn_vectors LIMIT 1",
                [],
                |row| row.get::<_, i64>(0).map(|n| n as usize / 4),
            )
            .ok();

        if let Some(dim) = existing_dim {
            if embedding.len() != dim {
                anyhow::bail!(
                    "embedding dimension mismatch: expected {dim}, got {} (session={session_id})",
                    embedding.len()
                );
            }
        }

        let bytes = floats_to_bytes(embedding);
        self.conn().execute(
            "INSERT INTO turn_vectors(session_id, turn_index, chunk_seq, model, embedded_at, embedding)
             VALUES (?1, ?2, ?3, ?4, datetime('now'), ?5)",
            rusqlite::params![session_id, turn_index as i64, chunk_seq as i64, model, bytes],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    fn search_vectors(
        &self,
        query_embedding: &[f32],
        limit: usize,
        session_ids: Option<&[String]>,
    ) -> crate::error::Result<Vec<VectorRow>> {
        let row_mapper = |row: &rusqlite::Row<'_>| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get::<_, i64>(2)? as u32,
                row.get::<_, i64>(3)? as u32,
                row.get(4)?,
            ))
        };

        let rows: Vec<(i64, String, u32, u32, Vec<u8>)> = if let Some(ids) = session_ids {
            if ids.is_empty() {
                return Ok(Vec::new());
            }
            let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{i}")).collect();
            let sql = format!(
                "SELECT id, session_id, turn_index, chunk_seq, embedding \
                 FROM turn_vectors WHERE session_id IN ({})",
                placeholders.join(",")
            );
            let mut stmt = self.conn().prepare(&sql)?;
            let collected: Vec<_> = stmt
                .query_map(rusqlite::params_from_iter(ids.iter()), row_mapper)?
                .filter_map(|r| r.ok())
                .collect();
            collected
        } else {
            let mut stmt = self.conn().prepare(
                "SELECT id, session_id, turn_index, chunk_seq, embedding FROM turn_vectors",
            )?;
            let collected: Vec<_> = stmt
                .query_map([], row_mapper)?
                .filter_map(|r| r.ok())
                .collect();
            collected
        };

        let mut scored: Vec<(f32, VectorRow)> = rows
            .into_iter()
            .map(|(id, session_id, turn_index, chunk_seq, bytes)| {
                let embedding = bytes_to_floats(&bytes);
                let distance = cosine_distance(query_embedding, &embedding);
                (
                    distance,
                    VectorRow {
                        rowid: id,
                        distance,
                        session_id,
                        turn_index,
                        chunk_seq,
                    },
                )
            })
            .collect();

        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        Ok(scored.into_iter().map(|(_, row)| row).collect())
    }

    fn get_vector_meta(&self, rowid: i64) -> anyhow::Result<(String, u32, u32)> {
        self.conn()
            .query_row(
                "SELECT session_id, turn_index, chunk_seq FROM turn_vectors WHERE id = ?1",
                [rowid],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)? as u32,
                        row.get::<_, i64>(2)? as u32,
                    ))
                },
            )
            .map_err(Into::into)
    }
}

pub(crate) fn floats_to_bytes(floats: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(floats.len() * 4);
    for f in floats {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

pub(crate) fn bytes_to_floats(bytes: &[u8]) -> Vec<f32> {
    if bytes.len() % 4 != 0 {
        tracing::warn!(
            blob_len = bytes.len(),
            "corrupt vector BLOB (not multiple of 4 bytes)"
        );
        return Vec::new();
    }
    bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect()
}

pub(crate) fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 1.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }
    1.0 - (dot / (norm_a * norm_b))
}

// ─── Additional Database methods (vector domain) ─────────────────────────────

use crate::error::Result;

impl Database {
    pub fn has_embeddings(&self) -> Result<bool> {
        let exists: i64 = self.conn().query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='turn_vectors'",
            [],
            |r| r.get(0),
        )?;
        if exists == 0 {
            return Ok(false);
        }
        let count: i64 = self
            .conn()
            .query_row("SELECT COUNT(*) FROM turn_vectors", [], |r| r.get(0))?;
        Ok(count > 0)
    }

    /// turn_vectors 테이블의 총 벡터 수. ANN stale 감지에 사용.
    pub fn count_vectors(&self) -> Result<usize> {
        let exists: i64 = self.conn().query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='turn_vectors'",
            [],
            |r| r.get(0),
        )?;
        if exists == 0 {
            return Ok(0);
        }
        let count: i64 = self
            .conn()
            .query_row("SELECT COUNT(*) FROM turn_vectors", [], |r| r.get(0))?;
        Ok(count as usize)
    }

    /// Sessions that have no rows in turn_vectors
    pub fn find_sessions_without_vectors(&self) -> Result<Vec<String>> {
        let table_exists: i64 = self.conn().query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='turn_vectors'",
            [],
            |r| r.get(0),
        )?;

        let query = if table_exists == 0 {
            "SELECT id FROM sessions"
        } else {
            "SELECT id FROM sessions WHERE id NOT IN (SELECT DISTINCT session_id FROM turn_vectors)"
        };

        let mut stmt = self.conn().prepare(query)?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Vector rows whose session_id does not exist in sessions
    pub fn find_orphan_vectors(&self) -> Result<Vec<(i64, String)>> {
        let table_exists: i64 = self.conn().query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='turn_vectors'",
            [],
            |r| r.get(0),
        )?;

        if table_exists == 0 {
            return Ok(Vec::new());
        }

        let mut stmt = self.conn().prepare(
            "SELECT id, session_id FROM turn_vectors WHERE session_id NOT IN (SELECT id FROM sessions)",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}
