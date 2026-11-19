//! Freeze pane.
#[derive(Debug, Clone, Default)]
pub struct FFreezePane {
    /// Top-left visible cell.
    pub top_left_cell: Option<String>,
    /// Number of frozen columns.
    pub x_split: Option<u32>,
    /// Number of frozen rows.
    pub y_split: Option<u32>,
    /// Active pane.
    pub active_pane: Option<String>,
    /// Raw `<pane>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
