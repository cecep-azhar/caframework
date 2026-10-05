//! Dual-level split architecture and tab grouping foundation models.
//!
//! Provides pure data structures and serialization for workspace layout trees,
//! macro viewport configurations (Level 1: Global Split), and micro pane splits (Level 2: In-Pane Split).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GlobalSplitLayout {
    Single,
    TwoColumns,
    TwoRows,
    TwoByTwoGrid,
}

impl Default for GlobalSplitLayout {
    fn default() -> Self {
        Self::Single
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InPaneSplitDirection {
    None,
    Horizontal,
    Vertical,
}

impl Default for InPaneSplitDirection {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaneRole {
    Primary,
    Secondary,
}

impl Default for PaneRole {
    fn default() -> Self {
        Self::Primary
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaneItem<T = serde_json::Value> {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<PaneRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<T>,
}

impl<T> PaneItem<T> {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            role: Some(PaneRole::Primary),
            payload: None,
        }
    }

    pub fn with_payload(mut self, payload: T) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn with_role(mut self, role: PaneRole) -> Self {
        self.role = Some(role);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabGroup<T = serde_json::Value> {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default)]
    pub split_direction: InPaneSplitDirection,
    pub panes: Vec<PaneItem<T>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_pane_id: Option<String>,
}

impl<T> TabGroup<T> {
    pub fn new(id: impl Into<String>, name: impl Into<String>, initial_pane: PaneItem<T>) -> Self {
        let initial_pane_id = initial_pane.id.clone();
        Self {
            id: id.into(),
            name: name.into(),
            color: None,
            split_direction: InPaneSplitDirection::None,
            panes: vec![initial_pane.with_role(PaneRole::Primary)],
            active_pane_id: Some(initial_pane_id),
        }
    }

    pub fn split_pane(&mut self, new_pane: PaneItem<T>, direction: InPaneSplitDirection) {
        self.split_direction = direction;
        let id = new_pane.id.clone();
        self.panes.push(new_pane.with_role(PaneRole::Secondary));
        self.active_pane_id = Some(id);
    }

    pub fn close_pane(&mut self, pane_id: &str) -> bool {
        self.panes.retain(|p| p.id != pane_id);
        if self.panes.len() <= 1 {
            self.split_direction = InPaneSplitDirection::None;
        }
        if self.active_pane_id.as_deref() == Some(pane_id) {
            self.active_pane_id = self.panes.first().map(|p| p.id.clone());
        }
        self.panes.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSnapshot<T = serde_json::Value> {
    pub id: String,
    pub name: String,
    pub global_layout: GlobalSplitLayout,
    pub groups: Vec<TabGroup<T>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_group_id: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_split_tree_lifecycle() {
        let mut group: TabGroup<String> = TabGroup::new(
            "grp-1",
            "Production Group",
            PaneItem::new("pane-1", "Main Terminal"),
        );

        assert_eq!(group.panes.len(), 1);
        assert_eq!(group.split_direction, InPaneSplitDirection::None);
        assert_eq!(group.panes[0].role, Some(PaneRole::Primary));

        // Split pane horizontally (Micro Level 2)
        group.split_pane(
            PaneItem::new("pane-2", "DB Logs").with_payload("postgres-logs".to_string()),
            InPaneSplitDirection::Horizontal,
        );

        assert_eq!(group.panes.len(), 2);
        assert_eq!(group.split_direction, InPaneSplitDirection::Horizontal);
        assert_eq!(group.panes[1].role, Some(PaneRole::Secondary));
        assert_eq!(group.active_pane_id, Some("pane-2".to_string()));

        // Snapshot serialization & deserialization (Level 1 + Level 2)
        let snapshot = WorkspaceSnapshot {
            id: "ws-1".to_string(),
            name: "Default Ops".to_string(),
            global_layout: GlobalSplitLayout::TwoColumns,
            groups: vec![group.clone()],
            active_group_id: Some("grp-1".to_string()),
            created_at: 1000,
            updated_at: 1000,
        };

        let json = serde_json::to_string(&snapshot).expect("serialize snapshot");
        assert!(json.contains("two-columns"));
        assert!(json.contains("horizontal"));

        let restored: WorkspaceSnapshot<String> =
            serde_json::from_str(&json).expect("deserialize snapshot");
        assert_eq!(restored.global_layout, GlobalSplitLayout::TwoColumns);
        assert_eq!(restored.groups.len(), 1);
        assert_eq!(
            restored.groups[0].split_direction,
            InPaneSplitDirection::Horizontal
        );

        // Close secondary pane resets split direction
        let is_empty = group.close_pane("pane-2");
        assert!(!is_empty);
        assert_eq!(group.panes.len(), 1);
        assert_eq!(group.split_direction, InPaneSplitDirection::None);
        assert_eq!(group.active_pane_id, Some("pane-1".to_string()));
    }
}
