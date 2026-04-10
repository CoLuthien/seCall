use anyhow::Result;
use secall_core::{
    graph::{build::build_graph, export::export_graph_json},
    store::{get_default_db_path, Database},
    vault::Config,
};

pub fn run_build(since: Option<&str>, force: bool) -> Result<()> {
    let config = Config::load_or_default();
    let db = Database::open(&get_default_db_path())?;

    if force {
        eprintln!("Clearing existing graph...");
    }
    eprintln!("Building knowledge graph...");

    let result = build_graph(&db, &config.vault.path, since, force)?;

    eprintln!(
        "  {} sessions processed, {} skipped.",
        result.sessions_processed, result.sessions_skipped
    );
    eprintln!(
        "  {} nodes, {} edges created.",
        result.nodes_created, result.edges_created
    );
    Ok(())
}

pub fn run_stats() -> Result<()> {
    let db = Database::open(&get_default_db_path())?;
    let stats = db.graph_stats()?;

    println!("Graph Statistics:");
    println!("  Nodes: {}", stats.node_count);
    println!("  Edges: {}", stats.edge_count);
    println!();

    println!("Nodes by type:");
    for (t, c) in &stats.nodes_by_type {
        println!("  {}: {}", t, c);
    }
    println!();

    println!("Edges by relation:");
    for (r, c) in &stats.edges_by_relation {
        println!("  {}: {}", r, c);
    }
    Ok(())
}

pub fn run_export() -> Result<()> {
    let config = Config::load_or_default();
    let db = Database::open(&get_default_db_path())?;

    let graph_dir = config.vault.path.join("graph");
    std::fs::create_dir_all(&graph_dir)?;

    let output_path = graph_dir.join("graph.json");
    export_graph_json(&db, &output_path)?;

    eprintln!("Exported to {}", output_path.display());
    Ok(())
}
