use crate::error::Result;
use crate::ingest::{Session, Turn};
use crate::search::bm25::SessionMeta;

pub trait SessionRepo {
    fn insert_session(&self, session: &Session) -> Result<()>;
    fn update_session_vault_path(&self, session_id: &str, vault_path: &str) -> Result<()>;
    fn insert_turn(&self, session_id: &str, turn: &Turn) -> Result<i64>;
    fn session_exists(&self, session_id: &str) -> Result<bool>;
    fn session_exists_by_prefix(&self, prefix: &str) -> Result<bool>;
    fn get_session_meta(&self, session_id: &str) -> Result<SessionMeta>;
    /// 세션이 존재하고 end_time이 NULL이면 true (아직 열린 세션)
    fn is_session_open(&self, session_id: &str) -> Result<bool>;
    /// 세션과 관련 데이터(turns, vectors) 삭제 — 오픈 세션 재인제스트 전 사용
    fn delete_session(&self, session_id: &str) -> Result<()>;
}
