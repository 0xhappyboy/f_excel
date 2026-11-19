//! Hyperlink related.
#[derive(Debug, Clone, Default)]
pub struct FHyperlink {
    /// Hyperlink reference.
    pub reference: String,
    /// Target URL.
    pub target: Option<String>,
    /// Display text.
    pub text: Option<String>,
    /// Tooltip text.
    pub tooltip: Option<String>,
    /// Raw `<hyperlink>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
