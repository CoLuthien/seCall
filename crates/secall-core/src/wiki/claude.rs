use std::path::PathBuf;

use async_trait::async_trait;

use super::WikiBackend;

pub struct ClaudeBackend {
    pub model: String,
    pub vault_path: PathBuf,
}

#[async_trait]
impl WikiBackend for ClaudeBackend {
    fn name(&self) -> &'static str {
        "claude"
    }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        use std::io::{BufRead, Write as _};
        use std::process::Stdio;

        if !crate::command_exists("claude") {
            anyhow::bail!(
                "Claude Code CLI not found in PATH. \
                 Install: https://docs.anthropic.com/claude-code"
            );
        }

        let model_id = match self.model.as_str() {
            "opus" => "claude-opus-4-6",
            _ => "claude-sonnet-4-6",
        };

        let mut child = std::process::Command::new("claude")
            .args(["-p", "--model", model_id])
            .arg("--allowedTools")
            .arg("mcp__secall__recall,mcp__secall__get,mcp__secall__status,mcp__secall__wiki_search,Read,Write,Edit,Glob,Grep")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .current_dir(&self.vault_path)
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(prompt.as_bytes())?;
        }

        let output = if let Some(stdout) = child.stdout.take() {
            let reader = std::io::BufReader::new(stdout);
            let mut lines = Vec::new();
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        eprintln!("  | {}", l);
                        lines.push(l);
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "failed to read claude stdout");
                        break;
                    }
                }
            }
            lines.join("\n")
        } else {
            String::new()
        };

        let status = child.wait()?;
        if !status.success() {
            anyhow::bail!("claude exited with code {:?}", status.code());
        }

        Ok(output)
    }
}
