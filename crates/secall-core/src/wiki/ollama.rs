use async_trait::async_trait;

use super::WikiBackend;

pub struct OllamaBackend {
    pub api_url: String,
    pub model: String,
    pub max_tokens: u32,
}

#[async_trait]
impl WikiBackend for OllamaBackend {
    fn name(&self) -> &'static str {
        "ollama"
    }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/api/generate", self.api_url))
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": prompt,
                "stream": false,
                "options": { "num_predict": self.max_tokens }
            }))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Ollama request failed: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error: {body}");
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Ollama response parse error: {}", e))?;

        json["response"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Ollama response missing 'response' field"))
    }
}
