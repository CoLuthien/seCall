use std::path::Path;

use crate::error::Result;
use crate::store::Database;

/// NetworkX node_link_data 호환 JSON으로 graph.json 내보내기
/// 원자적 쓰기: .json.tmp → rename
pub fn export_graph_json(db: &Database, output_path: &Path) -> Result<()> {
    // 노드 수집
    let node_rows = db.list_graph_nodes(None)?;
    let nodes: Vec<serde_json::Value> = node_rows
        .into_iter()
        .map(|(id, node_type, label)| {
            serde_json::json!({
                "id": id,
                "type": node_type,
                "label": label,
            })
        })
        .collect();

    // 엣지 수집
    let mut stmt = db.conn().prepare(
        "SELECT source, target, relation, confidence, weight FROM graph_edges",
    )?;
    let links: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, f64>(4)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .map(|(source, target, relation, confidence, weight)| {
            serde_json::json!({
                "source": source,
                "target": target,
                "relation": relation,
                "confidence": confidence,
                "weight": weight,
            })
        })
        .collect();

    let json = serde_json::json!({
        "directed": true,
        "multigraph": false,
        "nodes": nodes,
        "links": links,
    });

    let json_str = serde_json::to_string_pretty(&json)
        .map_err(|e| crate::SecallError::Other(e.into()))?;

    // 원자적 쓰기
    let tmp_path = output_path.with_extension("json.tmp");
    std::fs::write(&tmp_path, json_str)?;
    std::fs::rename(&tmp_path, output_path)?;

    Ok(())
}
