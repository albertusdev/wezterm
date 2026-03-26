use crate::cli::tab_resolution::resolve_tab_id;
use clap::Parser;
use mux::pane::PaneId;
use mux::tab::TabId;
use std::collections::HashMap;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct SetTabMetadata {
    /// Specify the target tab by its id
    #[arg(long, conflicts_with_all = &["pane_id"])]
    tab_id: Option<TabId>,
    /// Specify the current pane.
    /// The default is to use the current pane based on the
    /// environment variable WEZTERM_PANE.
    ///
    /// The pane is used to figure out which tab should be updated.
    #[arg(long)]
    pane_id: Option<PaneId>,

    /// One or more metadata assignments in KEY=VALUE form
    #[arg(required = true, value_name = "KEY=VALUE")]
    assignments: Vec<String>,
}

impl SetTabMetadata {
    pub async fn run(self, client: Client) -> anyhow::Result<()> {
        let tab_id = resolve_tab_id(&client, self.tab_id, self.pane_id).await?;
        let metadata = parse_assignments(self.assignments)?;

        client
            .set_tab_metadata(codec::TabMetadataChanged { tab_id, metadata })
            .await?;
        Ok(())
    }
}

fn parse_assignments(assignments: Vec<String>) -> anyhow::Result<HashMap<String, String>> {
    let mut metadata = HashMap::new();

    for assignment in assignments {
        let (key, value) = assignment
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("expected KEY=VALUE, got `{assignment}`"))?;
        if key.is_empty() {
            anyhow::bail!("expected non-empty metadata key in `{assignment}`");
        }
        metadata.insert(key.to_string(), value.to_string());
    }

    Ok(metadata)
}

#[cfg(test)]
mod test {
    use super::parse_assignments;

    #[test]
    fn parse_assignments_parses_values() {
        let parsed = parse_assignments(vec![
            "agent_hud.title=Focus".to_string(),
            "agent_hud.icon=!!".to_string(),
        ])
        .unwrap();

        assert_eq!(
            parsed.get("agent_hud.title").map(String::as_str),
            Some("Focus")
        );
        assert_eq!(parsed.get("agent_hud.icon").map(String::as_str), Some("!!"));
    }

    #[test]
    fn parse_assignments_rejects_missing_separator() {
        assert!(parse_assignments(vec!["broken".to_string()]).is_err());
    }
}
