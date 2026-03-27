use crate::cli::tab_resolution::{resolve_tab_id, resolve_tab_metadata};
use clap::Parser;
use mux::pane::PaneId;
use mux::tab::{TabId, TAB_METADATA_ACTIVITY, TAB_METADATA_ACTIVITY_COLOR};
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct SetTabActivity {
    /// Specify the target tab by its id
    #[arg(long, conflicts_with_all = &["pane_id"])]
    tab_id: Option<TabId>,
    /// Specify the current pane.
    /// The default is to use the current pane based on the
    /// environment variable WEZTERM_PANE.
    #[arg(long)]
    pane_id: Option<PaneId>,

    /// Compact activity marker to render in the tab rail
    #[arg(required = true)]
    activity: String,

    /// Optional activity color using any WezTerm-supported color string
    #[arg(long)]
    color: Option<String>,
}

impl SetTabActivity {
    pub async fn run(self, client: Client) -> anyhow::Result<()> {
        let tab_id = resolve_tab_id(&client, self.tab_id, self.pane_id).await?;
        let mut metadata = resolve_tab_metadata(&client, tab_id).await?;
        metadata.insert(TAB_METADATA_ACTIVITY.to_string(), self.activity);
        match self.color {
            Some(color) => {
                metadata.insert(TAB_METADATA_ACTIVITY_COLOR.to_string(), color);
            }
            None => {
                metadata.remove(TAB_METADATA_ACTIVITY_COLOR);
            }
        }
        client
            .set_tab_metadata(codec::TabMetadataChanged { tab_id, metadata })
            .await?;
        Ok(())
    }
}
