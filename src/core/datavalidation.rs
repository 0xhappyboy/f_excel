//! Data validation.
#[derive(Debug, Clone, Default)]
pub struct FDataValidation {
    /// Validation range.
    pub range: String,
    /// Validation type.
    pub validation_type: Option<String>,
    /// Operator.
    pub operator: Option<String>,
    /// Formula 1.
    pub formula1: Option<String>,
    /// Formula 2.
    pub formula2: Option<String>,
    /// Allow blank flag.
    pub allow_blank: Option<bool>,
    /// Show error message flag.
    pub show_error_message: Option<bool>,
    /// Show input message flag.
    pub show_input_message: Option<bool>,
    /// Raw `<dataValidation>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
