use crate::cli::tab_resolution::{resolve_tab_id, resolve_tab_metadata};
use clap::Parser;
use mux::pane::PaneId;
use mux::tab::{TabId, TAB_METADATA_ACCENT_COLOR};
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct ClearTabAccent {
    /// Specify the target tab by its id
    #[arg(long, conflicts_with_all = &["pane_id"])]
    tab_id: Option<TabId>,
    /// Specify the current pane.
    /// The default is to use the current pane based on the
    /// environment variable WEZTERM_PANE.
    #[arg(long)]
    pane_id: Option<PaneId>,
}

impl ClearTabAccent {
    pub async fn run(self, client: Client) -> anyhow::Result<()> {
        let tab_id = resolve_tab_id(&client, self.tab_id, self.pane_id).await?;
        let mut metadata = resolve_tab_metadata(&client, tab_id).await?;
        metadata.remove(TAB_METADATA_ACCENT_COLOR);
        client
            .set_tab_metadata(codec::TabMetadataChanged { tab_id, metadata })
            .await?;
        Ok(())
    }
}
