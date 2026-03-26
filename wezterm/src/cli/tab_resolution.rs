use mux::pane::PaneId;
use mux::tab::{PaneNode, TabId};
use std::collections::HashMap;
use wezterm_client::client::Client;

pub async fn resolve_tab_id(
    client: &Client,
    tab_id: Option<TabId>,
    pane_id: Option<PaneId>,
) -> anyhow::Result<TabId> {
    if let Some(tab_id) = tab_id {
        return Ok(tab_id);
    }

    let pane_id = client.resolve_pane_id(pane_id).await?;
    let panes = client.list_panes().await?;
    let pane_id_to_tab_id = build_pane_id_to_tab_id_map(panes.tabs);

    pane_id_to_tab_id
        .get(&pane_id)
        .copied()
        .ok_or_else(|| anyhow::anyhow!("unable to resolve current tab"))
}

pub async fn resolve_tab_metadata(
    client: &Client,
    tab_id: TabId,
) -> anyhow::Result<HashMap<String, String>> {
    let panes = client.list_panes().await?;

    for (idx, tabroot) in panes.tabs.iter().enumerate() {
        if let Some((_window_id, current_tab_id)) = tabroot.window_and_tab_ids() {
            if current_tab_id == tab_id {
                return Ok(panes.tab_metadata.get(idx).cloned().unwrap_or_default());
            }
        }
    }

    Err(anyhow::anyhow!(
        "unable to resolve tab metadata for tab {tab_id}"
    ))
}

fn build_pane_id_to_tab_id_map(tabs: Vec<PaneNode>) -> HashMap<PaneId, TabId> {
    let mut pane_id_to_tab_id = HashMap::new();

    for tabroot in tabs {
        let mut cursor = tabroot.into_tree().cursor();

        loop {
            if let Some(entry) = cursor.leaf_mut() {
                pane_id_to_tab_id.insert(entry.pane_id, entry.tab_id);
            }
            match cursor.preorder_next() {
                Ok(c) => cursor = c,
                Err(_) => break,
            }
        }
    }

    pane_id_to_tab_id
}
