use std::io::{Cursor, Read};
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::ingest::types::{Action, AgentKind, Role, Session, TokenUsage, Turn};
use crate::ingest::SessionParser;

/// UTF-8 safe한 바이트 위치 반환 (max_bytes 이하에서 char boundary)
fn truncate_utf8_safe(s: &str, max_bytes: usize) -> usize {
    if max_bytes >= s.len() {
        return s.len();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    end
}

// ── serde 구조체 ──────────────────────────────────────────────────────────────

/// conversations.json 최상위 — Vec<Conversation>
#[derive(Debug, Deserialize)]
struct Conversation {
    uuid: String,
    name: Option<String>,
    #[allow(dead_code)]
    summary: Option<String>,
    created_at: String,
    #[allow(dead_code)]
    updated_at: Option<String>,
    chat_messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    #[allow(dead_code)]
    uuid: String,
    text: Option<String>,
    content: Vec<ContentBlock>,
    sender: String, // "human" | "assistant"
    created_at: String,
    attachments: Option<Vec<Attachment>>,
    #[allow(dead_code)]
    files: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text {
        text: String,
        #[allow(dead_code)]
        citations: Option<Vec<serde_json::Value>>,
    },
    #[serde(rename = "thinking")]
    Thinking { thinking: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        name: String,
        input: Option<serde_json::Value>,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        #[allow(dead_code)]
        name: Option<String>,
        content: Option<Vec<serde_json::Value>>,
        #[allow(dead_code)]
        is_error: Option<bool>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
struct Attachment {
    file_name: Option<String>,
    #[allow(dead_code)]
    file_type: Option<String>,
    extracted_content: Option<String>,
}

// ── ZIP / JSON 읽기 ───────────────────────────────────────────────────────────

/// ZIP 파일이면 conversations.json을 추출, 아니면 그대로 읽기
fn read_conversations(path: &Path) -> crate::error::Result<Vec<Conversation>> {
    let data = std::fs::read(path)?;

    // ZIP 매직바이트 감지: PK\x03\x04
    let json_str = if data.starts_with(b"PK\x03\x04") {
        extract_conversations_from_zip(&data)?
    } else {
        String::from_utf8(data).map_err(|e| crate::SecallError::Parse {
            path: path.to_string_lossy().into_owned(),
            source: e.into(),
        })?
    };

    let conversations: Vec<Conversation> =
        serde_json::from_str(&json_str).map_err(|e| crate::SecallError::Parse {
            path: path.to_string_lossy().into_owned(),
            source: e.into(),
        })?;

    Ok(conversations)
}

fn extract_conversations_from_zip(data: &[u8]) -> crate::error::Result<String> {
    let reader = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| crate::SecallError::Parse {
        path: "<zip>".to_string(),
        source: e.into(),
    })?;

    let mut file =
        archive
            .by_name("conversations.json")
            .map_err(|e| crate::SecallError::Parse {
                path: "<zip>/conversations.json".to_string(),
                source: anyhow::anyhow!("conversations.json not found in ZIP: {e}"),
            })?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// ── Conversation → Session 변환 ───────────────────────────────────────────────

fn conversation_to_session(conv: &Conversation) -> crate::error::Result<Session> {
    let created = DateTime::parse_from_rfc3339(&conv.created_at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let last_msg_time = conv
        .chat_messages
        .last()
        .and_then(|m| DateTime::parse_from_rfc3339(&m.created_at).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let mut turns = Vec::new();

    for (i, msg) in conv.chat_messages.iter().enumerate() {
        let role = match msg.sender.as_str() {
            "human" => Role::User,
            "assistant" => Role::Assistant,
            _ => Role::System,
        };

        let timestamp = DateTime::parse_from_rfc3339(&msg.created_at)
            .ok()
            .map(|dt| dt.with_timezone(&Utc));

        let mut content_text = String::new();
        let mut thinking = None;
        let mut actions = Vec::new();

        for block in &msg.content {
            match block {
                ContentBlock::Text { text, .. } => {
                    if !content_text.is_empty() {
                        content_text.push('\n');
                    }
                    content_text.push_str(text);
                }
                ContentBlock::Thinking { thinking: t } => {
                    thinking = Some(t.clone());
                }
                ContentBlock::ToolUse { name, input } => {
                    let input_summary = input
                        .as_ref()
                        .map(|v| {
                            v.get("title")
                                .and_then(|t| t.as_str())
                                .unwrap_or_else(|| {
                                    v.get("query").and_then(|q| q.as_str()).unwrap_or("")
                                })
                                .to_string()
                        })
                        .unwrap_or_default();

                    actions.push(Action::ToolUse {
                        name: name.clone(),
                        input_summary,
                        output_summary: String::new(),
                        tool_use_id: None,
                    });
                }
                ContentBlock::ToolResult { content, .. } => {
                    if let Some(blocks) = content {
                        for b in blocks {
                            if let Some(text) = b.get("text").and_then(|t| t.as_str()) {
                                if !content_text.is_empty() {
                                    content_text.push('\n');
                                }
                                let end = truncate_utf8_safe(text, 500);
                                content_text.push_str(&text[..end]);
                            }
                        }
                    }
                }
                ContentBlock::Unknown => {}
            }
        }

        // 첨부파일의 extracted_content를 content에 추가
        if let Some(attachments) = &msg.attachments {
            for att in attachments {
                if let Some(extracted) = &att.extracted_content {
                    if !extracted.is_empty() {
                        content_text.push_str("\n\n[Attachment");
                        if let Some(fname) = &att.file_name {
                            content_text.push_str(&format!(": {fname}"));
                        }
                        content_text.push_str("]\n");
                        let end = truncate_utf8_safe(extracted, 2000);
                        content_text.push_str(&extracted[..end]);
                    }
                }
            }
        }

        // text 필드 fallback (content가 비어있으면)
        if content_text.is_empty() {
            if let Some(text) = &msg.text {
                content_text = text.clone();
            }
        }

        turns.push(Turn {
            index: i as u32,
            role,
            timestamp,
            content: content_text,
            actions,
            tokens: None,
            thinking,
            is_sidechain: false,
        });
    }

    let project = conv
        .name
        .as_ref()
        .filter(|n| !n.is_empty())
        .map(|n| sanitize_project_name(n));

    let host = Some(gethostname::gethostname().to_string_lossy().to_string());

    Ok(Session {
        id: conv.uuid.clone(),
        agent: AgentKind::ClaudeAi,
        model: None,
        project,
        cwd: None,
        git_branch: None,
        host,
        start_time: created,
        end_time: last_msg_time,
        turns,
        total_tokens: TokenUsage::default(),
        session_type: "interactive".to_string(),
    })
}

/// 대화 제목에서 vault 파일명에 안전한 프로젝트명 생성
fn sanitize_project_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    sanitized.trim().chars().take(50).collect()
}

// ── SessionParser 구현 ────────────────────────────────────────────────────────

pub struct ClaudeAiParser;

impl SessionParser for ClaudeAiParser {
    fn can_parse(&self, path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext == "zip" {
            return true;
        }
        if ext == "json" {
            if let Ok(data) = std::fs::read_to_string(path) {
                if data.trim_start().starts_with('[') {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                        if let Some(arr) = v.as_array() {
                            return arr
                                .first()
                                .map(|c| c["chat_messages"].is_array() && c["uuid"].is_string())
                                .unwrap_or(false);
                        }
                    }
                }
            }
        }
        false
    }

    fn parse(&self, path: &Path) -> crate::error::Result<Session> {
        let sessions = self.parse_all(path)?;
        sessions
            .into_iter()
            .next()
            .ok_or_else(|| crate::SecallError::Parse {
                path: path.to_string_lossy().into_owned(),
                source: anyhow::anyhow!("no conversations found"),
            })
    }

    fn parse_all(&self, path: &Path) -> crate::error::Result<Vec<Session>> {
        let conversations = read_conversations(path)?;

        let mut sessions = Vec::new();
        for conv in &conversations {
            if conv.chat_messages.is_empty() {
                continue;
            }
            match conversation_to_session(conv) {
                Ok(session) => sessions.push(session),
                Err(e) => {
                    tracing::warn!(
                        uuid = &conv.uuid,
                        name = conv.name.as_deref().unwrap_or("(unnamed)"),
                        error = %e,
                        "failed to parse conversation, skipping"
                    );
                }
            }
        }

        tracing::info!(
            total = conversations.len(),
            parsed = sessions.len(),
            "claude.ai conversations parsed"
        );

        Ok(sessions)
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::ClaudeAi
    }
}

// ── 테스트 ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_conversation() {
        let json = r#"[{
            "uuid": "test-uuid-001",
            "name": "테스트 대화",
            "created_at": "2026-04-01T10:00:00Z",
            "chat_messages": [
                {
                    "uuid": "msg-001",
                    "text": "안녕",
                    "content": [{"type": "text", "text": "안녕", "start_timestamp": null, "stop_timestamp": null, "flags": {}, "citations": []}],
                    "sender": "human",
                    "created_at": "2026-04-01T10:00:00Z",
                    "updated_at": "2026-04-01T10:00:00Z",
                    "attachments": [],
                    "files": []
                },
                {
                    "uuid": "msg-002",
                    "text": "안녕하세요!",
                    "content": [{"type": "text", "text": "안녕하세요!", "start_timestamp": null, "stop_timestamp": null, "flags": {}, "citations": []}],
                    "sender": "assistant",
                    "created_at": "2026-04-01T10:00:01Z",
                    "updated_at": "2026-04-01T10:00:01Z",
                    "attachments": [],
                    "files": []
                }
            ]
        }]"#;

        let convs: Vec<Conversation> = serde_json::from_str(json).unwrap();
        let session = conversation_to_session(&convs[0]).unwrap();

        assert_eq!(session.id, "test-uuid-001");
        assert_eq!(session.agent, AgentKind::ClaudeAi);
        assert_eq!(session.turns.len(), 2);
        assert_eq!(session.turns[0].role, Role::User);
        assert_eq!(session.turns[0].content, "안녕");
        assert_eq!(session.turns[1].role, Role::Assistant);
        assert!(session.project.is_some());
    }

    #[test]
    fn test_parse_empty_conversation_skipped() {
        let json = r#"[{
            "uuid": "empty-001",
            "name": "",
            "created_at": "2026-04-01T10:00:00Z",
            "chat_messages": []
        }]"#;

        let convs: Vec<Conversation> = serde_json::from_str(json).unwrap();
        // empty chat_messages 파싱 자체는 성공
        assert!(convs[0].chat_messages.is_empty());
    }

    #[test]
    fn test_parse_all_pretty_printed_multi_conversation() {
        // pretty-printed JSON: 앞 200바이트 안에 "chat_messages"가 없는 케이스.
        // 이전 is_claude_ai_json() 휴리스틱이면 is_multi=false → 첫 대화만 ingest되는 버그.
        // parse_all()이 모든 대화를 반환하는지 검증.
        let json = r#"[
  {
    "uuid": "conv-aaaaaa",
    "name": "Architecture design patterns in distributed systems",
    "summary": "A detailed discussion covering microservices and service mesh patterns",
    "created_at": "2026-04-01T10:00:00Z",
    "chat_messages": [
      {
        "uuid": "msg-001",
        "text": "Hello",
        "content": [
          {
            "type": "text",
            "text": "Hello",
            "start_timestamp": null,
            "stop_timestamp": null,
            "flags": {},
            "citations": []
          }
        ],
        "sender": "human",
        "created_at": "2026-04-01T10:00:00Z",
        "updated_at": "2026-04-01T10:00:00Z",
        "attachments": [],
        "files": []
      }
    ]
  },
  {
    "uuid": "conv-bbbbbb",
    "name": "Second conversation",
    "created_at": "2026-04-02T10:00:00Z",
    "chat_messages": [
      {
        "uuid": "msg-002",
        "text": "World",
        "content": [
          {
            "type": "text",
            "text": "World",
            "start_timestamp": null,
            "stop_timestamp": null,
            "flags": {},
            "citations": []
          }
        ],
        "sender": "human",
        "created_at": "2026-04-02T10:00:00Z",
        "updated_at": "2026-04-02T10:00:00Z",
        "attachments": [],
        "files": []
      }
    ]
  }
]"#;

        // "chat_messages"가 200바이트 이후에 등장하는지 확인
        let first_200 = &json[..truncate_utf8_safe(json, 200)];
        assert!(
            !first_200.contains("chat_messages"),
            "test fixture must push chat_messages beyond 200 bytes"
        );

        let dir = tempfile::tempdir().unwrap();
        let json_path = dir.path().join("conversations.json");
        std::fs::write(&json_path, json).unwrap();

        let parser = ClaudeAiParser;
        let sessions = parser.parse_all(&json_path).unwrap();

        assert_eq!(sessions.len(), 2, "both conversations must be parsed");
        assert_eq!(sessions[0].id, "conv-aaaaaa");
        assert_eq!(sessions[1].id, "conv-bbbbbb");
    }

    #[test]
    fn test_unknown_content_block_skipped() {
        let json = r#"[{
            "uuid": "test-002",
            "name": "Unknown blocks",
            "created_at": "2026-04-01T10:00:00Z",
            "chat_messages": [{
                "uuid": "msg-001",
                "text": "test",
                "content": [
                    {"type": "text", "text": "hello", "start_timestamp": null, "stop_timestamp": null, "flags": {}, "citations": []},
                    {"type": "voice_note", "title": "memo", "text": "voiced"}
                ],
                "sender": "human",
                "created_at": "2026-04-01T10:00:00Z",
                "updated_at": "2026-04-01T10:00:00Z",
                "attachments": [],
                "files": []
            }]
        }]"#;

        let convs: Vec<Conversation> = serde_json::from_str(json).unwrap();
        let session = conversation_to_session(&convs[0]).unwrap();
        assert_eq!(session.turns[0].content, "hello");
    }
}
