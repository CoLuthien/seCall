pub const CURRENT_SCHEMA_VERSION: u32 = 3;

pub const CREATE_SESSIONS: &str = "
CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY,
    agent       TEXT NOT NULL,
    model       TEXT,
    project     TEXT,
    cwd         TEXT,
    git_branch  TEXT,
    start_time  TEXT NOT NULL,
    end_time    TEXT,
    turn_count  INTEGER DEFAULT 0,
    tokens_in   INTEGER DEFAULT 0,
    tokens_out  INTEGER DEFAULT 0,
    tools_used  TEXT,
    tags        TEXT,
    vault_path  TEXT,
    host        TEXT,
    summary     TEXT,
    ingested_at TEXT NOT NULL,
    status      TEXT DEFAULT 'raw'
);
";

pub const CREATE_TURNS: &str = "
CREATE TABLE IF NOT EXISTS turns (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    turn_index  INTEGER NOT NULL,
    role        TEXT NOT NULL,
    timestamp   TEXT,
    content     TEXT NOT NULL,
    has_tool    INTEGER DEFAULT 0,
    tool_names  TEXT,
    thinking    TEXT,
    tokens_in   INTEGER DEFAULT 0,
    tokens_out  INTEGER DEFAULT 0,
    UNIQUE(session_id, turn_index)
);
";

pub const CREATE_TURNS_FTS: &str = "
CREATE VIRTUAL TABLE IF NOT EXISTS turns_fts USING fts5(
    content,
    session_id UNINDEXED,
    turn_id UNINDEXED,
    tokenize='unicode61'
);
";

pub const CREATE_INGEST_LOG: &str = "
CREATE TABLE IF NOT EXISTS ingest_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  TEXT NOT NULL,
    action      TEXT NOT NULL,
    timestamp   TEXT NOT NULL,
    details     TEXT
);
";

pub const CREATE_CONFIG: &str = "
CREATE TABLE IF NOT EXISTS config (
    key   TEXT PRIMARY KEY,
    value TEXT
);
";

pub const CREATE_INDEXES: &str = "
CREATE INDEX IF NOT EXISTS idx_turns_session ON turns(session_id);
CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project);
CREATE INDEX IF NOT EXISTS idx_sessions_agent ON sessions(agent);
CREATE INDEX IF NOT EXISTS idx_sessions_date ON sessions(start_time);
";

pub const CREATE_QUERY_CACHE: &str = "
CREATE TABLE IF NOT EXISTS query_cache (
    query_hash  TEXT PRIMARY KEY,
    original    TEXT NOT NULL,
    expanded    TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
";

pub const CREATE_GRAPH_NODES: &str = "
CREATE TABLE IF NOT EXISTS graph_nodes (
    id    TEXT PRIMARY KEY,
    type  TEXT NOT NULL,
    label TEXT NOT NULL,
    meta  TEXT
);
";

pub const CREATE_GRAPH_EDGES: &str = "
CREATE TABLE IF NOT EXISTS graph_edges (
    source     TEXT NOT NULL REFERENCES graph_nodes(id),
    target     TEXT NOT NULL REFERENCES graph_nodes(id),
    relation   TEXT NOT NULL,
    confidence TEXT NOT NULL DEFAULT 'EXTRACTED',
    weight     REAL DEFAULT 1.0,
    meta       TEXT,
    UNIQUE(source, target, relation)
);
";

pub const CREATE_GRAPH_INDEXES: &str = "
CREATE INDEX IF NOT EXISTS idx_graph_nodes_type ON graph_nodes(type);
CREATE INDEX IF NOT EXISTS idx_graph_edges_source ON graph_edges(source);
CREATE INDEX IF NOT EXISTS idx_graph_edges_target ON graph_edges(target);
CREATE INDEX IF NOT EXISTS idx_graph_edges_relation ON graph_edges(relation);
";
