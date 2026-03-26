use crate::cli::tab_resolution::resolve_tab_id;
use clap::Parser;
use mux::pane::PaneId;
use mux::tab::TabId;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct SetTabTitle {
    /// Specify the target tab by its id
    #[arg(long, conflicts_with_all=&["pane_id"])]
    tab_id: Option<TabId>,
    /// Specify the current pane.
    /// The default is to use the current pane based on the
    /// environment variable WEZTERM_PANE.
    ///
    /// The pane is used to figure out which tab should be renamed.
    #[arg(long)]
    pane_id: Option<PaneId>,

    /// The new title for the tab
    title: String,
}

impl SetTabTitle {
    pub async fn run(self, client: Client) -> anyhow::Result<()> {
        let tab_id = resolve_tab_id(&client, self.tab_id, self.pane_id).await?;

        client
            .set_tab_title(codec::TabTitleChanged {
                tab_id,
                title: self.title,
            })
            .await?;
        Ok(())
    }
}
