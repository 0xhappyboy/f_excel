//! Comment-related.
#[derive(Debug, Clone, Default)]
pub struct FComment {
    /// Comment reference.
    pub reference: String,
    /// Comment author.
    pub author: Option<String>,
    /// Comment text.
    pub text: String,
    /// Raw `<comment>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
