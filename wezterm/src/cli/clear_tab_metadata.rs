use crate::cli::tab_resolution::{resolve_tab_id, resolve_tab_metadata};
use clap::Parser;
use mux::pane::PaneId;
use mux::tab::TabId;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct ClearTabMetadata {
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

    /// One or more metadata keys to clear. Omit all keys to clear the entire metadata map.
    #[arg(value_name = "KEY")]
    keys: Vec<String>,
}

impl ClearTabMetadata {
    pub async fn run(self, client: Client) -> anyhow::Result<()> {
        let tab_id = resolve_tab_id(&client, self.tab_id, self.pane_id).await?;
        let mut metadata = resolve_tab_metadata(&client, tab_id).await?;

        if self.keys.is_empty() {
            metadata.clear();
        } else {
            for key in self.keys {
                metadata.remove(&key);
            }
        }

        client
            .set_tab_metadata(codec::TabMetadataChanged { tab_id, metadata })
            .await?;
        Ok(())
    }
}
