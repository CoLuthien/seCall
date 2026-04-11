use std::path::PathBuf;

use anyhow::Result;
use secall_core::vault::Config;

pub async fn run_update(
    model: &str,
    backend: Option<&str>,
    since: Option<&str>,
    session: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    // 1. wiki/ directory check
    let config = Config::load_or_default();
    let wiki_dir = config.vault.path.join("wiki");
    if !wiki_dir.exists() {
        anyhow::bail!("wiki/ directory not found. Run `secall init` first.");
    }

    // 2. Load prompt
    let prompt = if let Some(sid) = session {
        load_incremental_prompt(sid)?
    } else {
        load_batch_prompt(since)?
    };

    // 3. dry-run: print prompt and exit
    if dry_run {
        println!("{prompt}");
        return Ok(());
    }

    // 4. 백엔드 선택: --backend 플래그 → config wiki.default_backend → "claude"
    let backend_name = backend
        .map(|s| s.to_string())
        .unwrap_or_else(|| config.wiki.default_backend.clone());

    let target = if let Some(sid) = session {
        format!("session {}", &sid[..sid.len().min(8)])
    } else {
        "all sessions".to_string()
    };
    eprintln!("Wiki update: {} (backend: {})", target, backend_name);

    // 5. WikiBackend 인스턴스 생성
    let backend_box: Box<dyn secall_core::wiki::WikiBackend> = match backend_name.as_str() {
        "ollama" => {
            let cfg = config.wiki_backend_config("ollama");
            Box::new(secall_core::wiki::OllamaBackend {
                api_url: cfg
                    .api_url
                    .unwrap_or_else(|| "http://localhost:11434".to_string()),
                model: cfg.model.unwrap_or_else(|| "llama3".to_string()),
                max_tokens: cfg.max_tokens,
            })
        }
        "lmstudio" => {
            let cfg = config.wiki_backend_config("lmstudio");
            Box::new(secall_core::wiki::LmStudioBackend {
                api_url: cfg
                    .api_url
                    .unwrap_or_else(|| "http://localhost:1234".to_string()),
                model: cfg.model.unwrap_or_else(|| "local-model".to_string()),
                max_tokens: cfg.max_tokens,
            })
        }
        "claude" => Box::new(secall_core::wiki::ClaudeBackend {
            model: model.to_string(),
            vault_path: config.vault.path.clone(),
        }),
        _ => {
            anyhow::bail!(
                "Unknown backend '{}'. Supported: claude, ollama, lmstudio",
                backend_name
            );
        }
    };

    // 6. 생성 실행
    eprintln!("  Launching {}...", backend_box.name());
    let output = backend_box.generate(&prompt).await?;

    eprintln!("  ✓ Wiki update complete.");
    if !output.trim().is_empty() {
        tracing::debug!(
            output_len = output.len(),
            backend = backend_name,
            "wiki backend produced output"
        );
    }

    Ok(())
}

pub fn run_status() -> Result<()> {
    let config = Config::load_or_default();
    let wiki_dir = config.vault.path.join("wiki");

    if !wiki_dir.exists() {
        println!("Wiki not initialized. Run `secall init`.");
        return Ok(());
    }

    let mut page_count = 0;
    for entry in walkdir::WalkDir::new(&wiki_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
            page_count += 1;
        }
    }

    println!("Wiki: {}", wiki_dir.display());
    println!("Pages: {page_count}");
    Ok(())
}

fn load_batch_prompt(since: Option<&str>) -> Result<String> {
    let custom_path = prompt_dir().join("wiki-update.md");
    let mut prompt = if custom_path.exists() {
        std::fs::read_to_string(&custom_path)?
    } else {
        include_str!("../../../../docs/prompts/wiki-update.md").to_string()
    };

    if let Some(since) = since {
        prompt.push_str(&format!(
            "\n\n## 추가 조건\n- `--since {since}` 이후 세션만 검색하세요.\n"
        ));
    }

    Ok(prompt)
}

fn load_incremental_prompt(session_id: &str) -> Result<String> {
    let custom_path = prompt_dir().join("wiki-incremental.md");
    let template = if custom_path.exists() {
        std::fs::read_to_string(&custom_path)?
    } else {
        include_str!("../../../../docs/prompts/wiki-incremental.md").to_string()
    };

    Ok(template
        .replace("{SECALL_SESSION_ID}", session_id)
        .replace(
            "{SECALL_AGENT}",
            &std::env::var("SECALL_AGENT").unwrap_or_default(),
        )
        .replace(
            "{SECALL_PROJECT}",
            &std::env::var("SECALL_PROJECT").unwrap_or_default(),
        )
        .replace(
            "{SECALL_DATE}",
            &std::env::var("SECALL_DATE").unwrap_or_default(),
        ))
}

fn prompt_dir() -> PathBuf {
    if let Ok(p) = std::env::var("SECALL_PROMPTS_DIR") {
        return PathBuf::from(p);
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("secall")
        .join("prompts")
}
