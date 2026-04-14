use anyhow::Result;

#[derive(Debug, serde::Deserialize)]
pub struct ReviewResult {
    #[serde(default)]
    pub issues: Vec<ReviewIssue>,
    #[serde(default)]
    pub approved: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct ReviewIssue {
    pub severity: String,
    pub description: String,
    pub suggestion: Option<String>,
}

/// 위키 페이지를 Sonnet/Opus에 보내 검수
pub async fn review_page(
    api_key: &str,
    model: &str,
    page_content: &str,
    source_summary: &str,
) -> Result<ReviewResult> {
    let system_prompt = load_review_system_prompt();

    let user_prompt = format!(
        "## 위키 페이지 내용\n\n{}\n\n## 원본 세션 요약\n\n{}",
        page_content, source_summary
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let model_id = match model {
        "opus" => "claude-opus-4-6",
        _ => "claude-sonnet-4-6",
    };

    let payload = serde_json::json!({
        "model": model_id,
        "max_tokens": 2048,
        "system": system_prompt,
        "messages": [
            {"role": "user", "content": user_prompt}
        ]
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Review API request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Review API error {}: {}", status, body);
    }

    let json: serde_json::Value = resp.json().await?;
    let text = json["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|block| block["text"].as_str())
        .unwrap_or("{}");

    parse_review_response(text)
}

/// 검수 응답 텍스트를 ReviewResult로 파싱
pub fn parse_review_response(text: &str) -> Result<ReviewResult> {
    // JSON 블록을 찾아서 파싱 (```json ... ``` 또는 직접 JSON)
    let json_str = extract_json_block(text);

    serde_json::from_str::<ReviewResult>(&json_str).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse review response as JSON: {}. Raw: {}",
            e,
            &text[..text.len().min(200)]
        )
    })
}

/// 텍스트에서 JSON 블록 추출
fn extract_json_block(text: &str) -> String {
    // ```json ... ``` 블록 찾기
    if let Some(start) = text.find("```json") {
        let after = &text[start + 7..];
        if let Some(end) = after.find("```") {
            return after[..end].trim().to_string();
        }
    }
    // { ... } 직접 찾기
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return text[start..=end].to_string();
        }
    }
    text.to_string()
}

fn load_review_system_prompt() -> String {
    let custom_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("secall")
        .join("prompts")
        .join("wiki-review.md");

    if custom_path.exists() {
        std::fs::read_to_string(&custom_path).unwrap_or_default()
    } else {
        include_str!("../../../../docs/prompts/wiki-review.md").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_review_approved() {
        let text = r#"{"issues": [], "approved": true}"#;
        let result = parse_review_response(text).unwrap();
        assert!(result.approved);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_parse_review_with_issues() {
        let text = r#"{"issues": [{"severity": "warning", "description": "Missing code snippet", "suggestion": "Add the code"}], "approved": false}"#;
        let result = parse_review_response(text).unwrap();
        assert!(!result.approved);
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].severity, "warning");
    }

    #[test]
    fn test_parse_review_json_in_codeblock() {
        let text = "Here is my review:\n```json\n{\"issues\": [], \"approved\": true}\n```\nDone.";
        let result = parse_review_response(text).unwrap();
        assert!(result.approved);
    }

    #[test]
    fn test_extract_json_block_direct() {
        let text = r#"Some text {"issues": []} end"#;
        let json = extract_json_block(text);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }
}
