use async_trait::async_trait;

use super::WikiBackend;

pub struct HaikuBackend {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub system_prompt: String,
}

#[async_trait]
impl WikiBackend for HaikuBackend {
    fn name(&self) -> &'static str {
        "haiku"
    }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?;

        let payload = serde_json::json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "system": self.system_prompt,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        });

        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Anthropic API request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error {}: {}", status, body);
        }

        let json: serde_json::Value = resp.json().await?;

        // content[0].text 추출
        json["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|block| block["text"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Unexpected Anthropic API response format"))
    }
}

impl HaikuBackend {
    /// ANTHROPIC_API_KEY 환경변수에서 키를 읽어 생성
    pub fn from_env(
        model: Option<String>,
        max_tokens: u32,
        system_prompt: String,
    ) -> anyhow::Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "ANTHROPIC_API_KEY environment variable not set. \
                 Get your key at https://console.anthropic.com/"
            )
        })?;

        Ok(Self {
            api_key,
            model: model.unwrap_or_else(|| "claude-haiku-4-5-20251001".to_string()),
            max_tokens,
            system_prompt,
        })
    }
}
