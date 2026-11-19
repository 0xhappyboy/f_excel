//! Conditional formatting.
#[derive(Debug, Clone, Default)]
pub struct FConditionalFormat {
    /// Format range.
    pub range: String,
    /// Format type.
    pub format_type: Option<String>,
    /// Priority.
    pub priority: Option<u32>,
    /// Operator.
    pub operator: Option<String>,
    /// Formula.
    pub formula: Option<String>,
    /// Dxf style index.
    pub dxf_id: Option<u32>,
    /// Raw `<conditionalFormatting>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
