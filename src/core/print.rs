/// Print settings.
#[derive(Debug, Clone, Default)]
pub struct FPrintSettings {
    /// Paper size.
    pub paper_size: Option<u32>,
    /// Orientation.
    pub orientation: Option<String>,
    /// Fit to width.
    pub fit_to_width: Option<u32>,
    /// Fit to height.
    pub fit_to_height: Option<u32>,
    /// Print area.
    pub print_area: Option<String>,
    /// Print titles.
    pub print_titles: Option<String>,
    /// Raw `<pageSetup>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
