use anyhow::Result;
use secall_core::{store::get_default_db_path, store::Database, vault::Config};

pub async fn run_backfill(dry_run: bool) -> Result<()> {
    let config = Config::load_or_default();
    let classification = &config.ingest.classification;

    if classification.rules.is_empty() {
        eprintln!(
            "No classification rules found in config. \
             Add [[ingest.classification.rules]] to .secall.toml"
        );
        return Ok(());
    }

    let db_path = get_default_db_path();
    let db = Database::open(&db_path)?;

    // 전체 세션: (id, cwd, project, agent, first_user_content)
    let sessions = db.get_all_sessions_for_classify()?;

    let total = sessions.len();
    let mut updated = 0usize;

    let compiled_rules: Vec<(regex::Regex, String)> = classification
        .rules
        .iter()
        .map(|rule| {
            regex::Regex::new(&rule.pattern)
                .map(|re| (re, rule.session_type.clone()))
                .map_err(|e| anyhow::anyhow!("invalid regex pattern {:?}: {}", rule.pattern, e))
        })
        .collect::<anyhow::Result<_>>()?;

    for (session_id, _cwd, _project, _agent, first_content) in &sessions {
        let new_type = super::ingest::apply_classification(
            &compiled_rules,
            first_content,
            &classification.default,
        );

        let short_id = &session_id[..8.min(session_id.len())];
        if dry_run {
            eprintln!("  [dry-run] {} → {}", short_id, new_type);
        } else {
            db.update_session_type(session_id, &new_type)?;
            tracing::debug!(session = short_id, session_type = new_type, "classified");
        }
        updated += 1;
    }

    eprintln!(
        "Classify {}complete: {}/{} sessions processed",
        if dry_run { "(dry-run) " } else { "" },
        updated,
        total,
    );
    Ok(())
}
