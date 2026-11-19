use crate::core::docs::{FCell, FRow, FSheet, FWorkbook};
use crate::core::sheet::{FCellAlignment, FCellStyle, FCellValue};
use crate::encode_xml_text;
use std::io::Write;
use zip::ZipWriter;
use zip::write::FileOptions;
/// XML declaration shared by every serialized part.
const XML_DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";
/// Package root relationships part.
const ROOT_RELS_XML: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
    "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
    "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/>",
    "</Relationships>"
);
/// Serializes a complete Excel workbook to bytes.
pub(crate) fn serialize_workbook(wb: &FWorkbook) -> crate::FExcelResult<Vec<u8>> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options: FileOptions<'_, '_, ()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    let has_styles = !wb.styles.is_empty();
    let has_shared_strings = wb
        .sheets
        .iter()
        .flat_map(|s| s.rows.iter())
        .flat_map(|r| r.cells.iter())
        .any(|c| matches!(c.value, FCellValue::String(_)));
    let has_theme = wb.theme.is_some();
    writer.start_file("[Content_Types].xml", options)?;
    writer.write_all(
        serialize_content_types(wb, has_styles, has_shared_strings, has_theme).as_bytes(),
    )?;
    writer.start_file("_rels/.rels", options)?;
    writer.write_all(ROOT_RELS_XML.as_bytes())?;
    let main_part = if wb.package.main_part.trim().is_empty() {
        "xl/workbook.xml"
    } else {
        wb.package.main_part.as_str()
    };
    writer.start_file(main_part, options)?;
    writer.write_all(serialize_workbook_xml(wb).as_bytes())?;
    writer.start_file("xl/_rels/workbook.xml.rels", options)?;
    writer.write_all(serialize_workbook_rels(wb).as_bytes())?;
    for (idx, sheet) in wb.sheets.iter().enumerate() {
        let path = if sheet.path.trim().is_empty() {
            format!("xl/worksheets/sheet{}.xml", idx + 1)
        } else {
            sheet.path.clone()
        };
        writer.start_file(path.as_str(), options)?;
        writer.write_all(serialize_sheet(sheet).as_bytes())?;
    }
    if has_shared_strings {
        writer.start_file("xl/sharedStrings.xml", options)?;
        writer.write_all(serialize_shared_strings(wb).as_bytes())?;
    }
    if has_styles {
        writer.start_file("xl/styles.xml", options)?;
        writer.write_all(serialize_styles(&wb.styles).as_bytes())?;
    }
    if let Some(theme) = &wb.theme {
        writer.start_file("xl/theme/theme1.xml", options)?;
        writer.write_all(theme.as_bytes())?;
    }
    for part in &wb.package.parts {
        if part.path == "[Content_Types].xml"
            || part.path == "_rels/.rels"
            || part.path == main_part
            || part.path == "xl/_rels/workbook.xml.rels"
            || part.path.starts_with("xl/worksheets/")
            || part.path == "xl/sharedStrings.xml"
            || part.path == "xl/styles.xml"
            || part.path == "xl/theme/theme1.xml"
        {
            continue;
        }
        writer.start_file(part.path.as_str(), options)?;
        writer.write_all(&part.data)?;
    }
    let cursor = writer.finish()?;
    Ok(cursor.into_inner())
}
/// Serializes the package content types part.
fn serialize_content_types(
    wb: &FWorkbook,
    has_styles: bool,
    has_shared_strings: bool,
    has_theme: bool,
) -> String {
    let mut out = String::new();
    out.push_str(XML_DECL);
    out.push_str("<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">");
    out.push_str("<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>");
    out.push_str("<Default Extension=\"xml\" ContentType=\"application/xml\"/>");
    out.push_str("<Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>");
    for (idx, sheet) in wb.sheets.iter().enumerate() {
        let path = if sheet.path.trim().is_empty() {
            format!("/xl/worksheets/sheet{}.xml", idx + 1)
        } else {
            format!("/{}", sheet.path.trim_start_matches('/'))
        };
        out.push_str(&format!(
            "<Override PartName=\"{}\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>",
            encode_xml_text(&path)
        ));
    }
    if has_shared_strings {
        out.push_str("<Override PartName=\"/xl/sharedStrings.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml\"/>");
    }
    if has_styles {
        out.push_str("<Override PartName=\"/xl/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml\"/>");
    }
    if has_theme {
        out.push_str("<Override PartName=\"/xl/theme/theme1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.theme+xml\"/>");
    }
    out.push_str("</Types>");
    out
}
/// Serializes workbook.xml.
fn serialize_workbook_xml(wb: &FWorkbook) -> String {
    let mut out = String::new();
    out.push_str(XML_DECL);
    out.push_str("<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">");
    if wb.properties.code_name.is_some() || wb.properties.date1904.is_some() {
        out.push_str("<workbookPr");
        if let Some(code_name) = &wb.properties.code_name {
            out.push_str(&format!(" codeName=\"{}\"", encode_xml_text(code_name)));
        }
        if let Some(date1904) = wb.properties.date1904 {
            out.push_str(&format!(
                " date1904=\"{}\"",
                if date1904 { "1" } else { "0" }
            ));
        }
        out.push_str("/>");
    }
    if wb.view.active_tab.is_some()
        || wb.view.show_horizontal_scroll.is_some()
        || wb.view.show_vertical_scroll.is_some()
        || wb.view.show_sheet_tabs.is_some()
    {
        out.push_str("<bookViews><workbookView");
        if let Some(active_tab) = wb.view.active_tab {
            out.push_str(&format!(" activeTab=\"{}\"", active_tab));
        }
        if let Some(show_horizontal_scroll) = wb.view.show_horizontal_scroll {
            out.push_str(&format!(
                " showHorizontalScroll=\"{}\"",
                if show_horizontal_scroll { "1" } else { "0" }
            ));
        }
        if let Some(show_vertical_scroll) = wb.view.show_vertical_scroll {
            out.push_str(&format!(
                " showVerticalScroll=\"{}\"",
                if show_vertical_scroll { "1" } else { "0" }
            ));
        }
        if let Some(show_sheet_tabs) = wb.view.show_sheet_tabs {
            out.push_str(&format!(
                " showSheetTabs=\"{}\"",
                if show_sheet_tabs { "1" } else { "0" }
            ));
        }
        out.push_str("/></bookViews>");
    }
    out.push_str("<sheets>");
    for (idx, sheet) in wb.sheets.iter().enumerate() {
        out.push_str(&format!(
            "<sheet name=\"{}\" sheetId=\"{}\" r:id=\"rId{}\"/>",
            encode_xml_text(&sheet.name),
            idx + 1,
            idx + 2
        ));
    }
    out.push_str("</sheets>");
    if !wb.defined_names.is_empty() {
        out.push_str("<definedNames>");
        for dn in &wb.defined_names {
            out.push_str(&format!(
                "<definedName name=\"{}\"{}>{}</definedName>",
                encode_xml_text(&dn.name),
                if let Some(local_sheet_id) = dn.local_sheet_id {
                    format!(" localSheetId=\"{}\"", local_sheet_id)
                } else {
                    String::new()
                },
                encode_xml_text(&dn.text)
            ));
        }
        out.push_str("</definedNames>");
    }
    out.push_str("</workbook>");
    out
}
/// Serializes workbook relationships.
fn serialize_workbook_rels(wb: &FWorkbook) -> String {
    let mut out = String::new();
    out.push_str(XML_DECL);
    out.push_str(
        "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
    );
    out.push_str("<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>");
    for (idx, sheet) in wb.sheets.iter().enumerate() {
        let path = if sheet.path.trim().is_empty() {
            format!("worksheets/sheet{}.xml", idx + 1)
        } else {
            sheet.path.trim_start_matches("xl/").to_string()
        };
        out.push_str(&format!(
            "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"{}\"/>",
            idx + 2,
            encode_xml_text(&path)
        ));
    }
    out.push_str("</Relationships>");
    out
}
/// Serializes a worksheet.
fn serialize_sheet(sheet: &FSheet) -> String {
    let mut out = String::new();
    out.push_str(XML_DECL);
    out.push_str("<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">");
    if let Some(freeze) = &sheet.freeze {
        out.push_str("<sheetViews><sheetView workbookViewId=\"0\">");
        out.push_str("<pane");
        if let Some(x_split) = freeze.x_split {
            out.push_str(&format!(" xSplit=\"{}\"", x_split));
        }
        if let Some(y_split) = freeze.y_split {
            out.push_str(&format!(" ySplit=\"{}\"", y_split));
        }
        if let Some(top_left_cell) = &freeze.top_left_cell {
            out.push_str(&format!(
                " topLeftCell=\"{}\"",
                encode_xml_text(top_left_cell)
            ));
        }
        if let Some(active_pane) = &freeze.active_pane {
            out.push_str(&format!(" activePane=\"{}\"", encode_xml_text(active_pane)));
        }
        out.push_str("/>");
        out.push_str("</sheetView></sheetViews>");
    }
    if !sheet.column_widths.is_empty() {
        out.push_str("<cols>");
        for (idx, width) in &sheet.column_widths {
            out.push_str(&format!(
                "<col min=\"{}\" max=\"{}\" width=\"{}\" customWidth=\"1\"/>",
                idx, idx, width
            ));
        }
        out.push_str("</cols>");
    }
    out.push_str("<sheetData>");
    for row in &sheet.rows {
        out.push_str(&serialize_row(row));
    }
    out.push_str("</sheetData>");
    if let Some(af) = &sheet.autofilter {
        out.push_str(&format!(
            "<autoFilter ref=\"{}\">",
            encode_xml_text(&af.range)
        ));
        if !af.columns.is_empty() {
            out.push_str("<filterColumns>");
            for col in &af.columns {
                out.push_str(&format!("<filterColumn colId=\"{}\">", col.col_id));
                if !col.filters.is_empty() {
                    out.push_str("<filters>");
                    for f in &col.filters {
                        out.push_str(&format!("<filter val=\"{}\"/>", encode_xml_text(f)));
                    }
                    out.push_str("</filters>");
                }
                out.push_str("</filterColumn>");
            }
            out.push_str("</filterColumns>");
        }
        out.push_str("</autoFilter>");
    }
    if !sheet.merged_cells.is_empty() {
        out.push_str(&format!(
            "<mergeCells count=\"{}\">",
            sheet.merged_cells.len()
        ));
        for mc in &sheet.merged_cells {
            out.push_str(&format!("<mergeCell ref=\"{}\"/>", encode_xml_text(mc)));
        }
        out.push_str("</mergeCells>");
    }
    if !sheet.data_validations.is_empty() {
        out.push_str(&format!(
            "<dataValidations count=\"{}\">",
            sheet.data_validations.len()
        ));
        for dv in &sheet.data_validations {
            out.push_str(&format!(
                "<dataValidation sqref=\"{}\"",
                encode_xml_text(&dv.range)
            ));
            if let Some(validation_type) = &dv.validation_type {
                out.push_str(&format!(" type=\"{}\"", encode_xml_text(validation_type)));
            }
            if let Some(operator) = &dv.operator {
                out.push_str(&format!(" operator=\"{}\"", encode_xml_text(operator)));
            }
            if let Some(allow_blank) = dv.allow_blank {
                out.push_str(&format!(
                    " allowBlank=\"{}\"",
                    if allow_blank { "1" } else { "0" }
                ));
            }
            if let Some(show_error_message) = dv.show_error_message {
                out.push_str(&format!(
                    " showErrorMessage=\"{}\"",
                    if show_error_message { "1" } else { "0" }
                ));
            }
            if let Some(show_input_message) = dv.show_input_message {
                out.push_str(&format!(
                    " showInputMessage=\"{}\"",
                    if show_input_message { "1" } else { "0" }
                ));
            }
            out.push('>');
            if let Some(f1) = &dv.formula1 {
                out.push_str(&format!("<formula1>{}</formula1>", encode_xml_text(f1)));
            }
            if let Some(f2) = &dv.formula2 {
                out.push_str(&format!("<formula2>{}</formula2>", encode_xml_text(f2)));
            }
            out.push_str("</dataValidation>");
        }
        out.push_str("</dataValidations>");
    }
    if !sheet.conditional_formats.is_empty() {
        for cf in &sheet.conditional_formats {
            out.push_str(&format!(
                "<conditionalFormatting sqref=\"{}\">",
                encode_xml_text(&cf.range)
            ));
            out.push_str(&format!("<cfRule"));
            if let Some(format_type) = &cf.format_type {
                out.push_str(&format!(" type=\"{}\"", encode_xml_text(format_type)));
            }
            if let Some(priority) = cf.priority {
                out.push_str(&format!(" priority=\"{}\"", priority));
            }
            if let Some(operator) = &cf.operator {
                out.push_str(&format!(" operator=\"{}\"", encode_xml_text(operator)));
            }
            if let Some(dxf_id) = cf.dxf_id {
                out.push_str(&format!(" dxfId=\"{}\"", dxf_id));
            }
            out.push('>');
            if let Some(formula) = &cf.formula {
                out.push_str(&format!("<formula>{}</formula>", encode_xml_text(formula)));
            }
            out.push_str("</cfRule>");
            out.push_str("</conditionalFormatting>");
        }
    }
    if let Some(print) = &sheet.print {
        out.push_str("<pageSetup");
        if let Some(paper_size) = print.paper_size {
            out.push_str(&format!(" paperSize=\"{}\"", paper_size));
        }
        if let Some(orientation) = &print.orientation {
            out.push_str(&format!(
                " orientation=\"{}\"",
                encode_xml_text(orientation)
            ));
        }
        if let Some(fit_to_width) = print.fit_to_width {
            out.push_str(&format!(" fitToWidth=\"{}\"", fit_to_width));
        }
        if let Some(fit_to_height) = print.fit_to_height {
            out.push_str(&format!(" fitToHeight=\"{}\"", fit_to_height));
        }
        out.push_str("/>");
    }
    if let Some(protection) = &sheet.protection {
        out.push_str("<sheetProtection");
        if let Some(password) = &protection.password {
            out.push_str(&format!(" password=\"{}\"", encode_xml_text(password)));
        }
        if let Some(sheet) = protection.sheet {
            out.push_str(&format!(" sheet=\"{}\"", if sheet { "1" } else { "0" }));
        }
        if let Some(objects) = protection.objects {
            out.push_str(&format!(" objects=\"{}\"", if objects { "1" } else { "0" }));
        }
        if let Some(scenarios) = protection.scenarios {
            out.push_str(&format!(
                " scenarios=\"{}\"",
                if scenarios { "1" } else { "0" }
            ));
        }
        if let Some(format_cells) = protection.format_cells {
            out.push_str(&format!(
                " formatCells=\"{}\"",
                if format_cells { "1" } else { "0" }
            ));
        }
        if let Some(format_columns) = protection.format_columns {
            out.push_str(&format!(
                " formatColumns=\"{}\"",
                if format_columns { "1" } else { "0" }
            ));
        }
        if let Some(format_rows) = protection.format_rows {
            out.push_str(&format!(
                " formatRows=\"{}\"",
                if format_rows { "1" } else { "0" }
            ));
        }
        if let Some(insert_columns) = protection.insert_columns {
            out.push_str(&format!(
                " insertColumns=\"{}\"",
                if insert_columns { "1" } else { "0" }
            ));
        }
        if let Some(insert_rows) = protection.insert_rows {
            out.push_str(&format!(
                " insertRows=\"{}\"",
                if insert_rows { "1" } else { "0" }
            ));
        }
        if let Some(insert_hyperlinks) = protection.insert_hyperlinks {
            out.push_str(&format!(
                " insertHyperlinks=\"{}\"",
                if insert_hyperlinks { "1" } else { "0" }
            ));
        }
        if let Some(delete_columns) = protection.delete_columns {
            out.push_str(&format!(
                " deleteColumns=\"{}\"",
                if delete_columns { "1" } else { "0" }
            ));
        }
        if let Some(delete_rows) = protection.delete_rows {
            out.push_str(&format!(
                " deleteRows=\"{}\"",
                if delete_rows { "1" } else { "0" }
            ));
        }
        if let Some(select_locked_cells) = protection.select_locked_cells {
            out.push_str(&format!(
                " selectLockedCells=\"{}\"",
                if select_locked_cells { "1" } else { "0" }
            ));
        }
        if let Some(sort) = protection.sort {
            out.push_str(&format!(" sort=\"{}\"", if sort { "1" } else { "0" }));
        }
        if let Some(auto_filter) = protection.auto_filter {
            out.push_str(&format!(
                " autoFilter=\"{}\"",
                if auto_filter { "1" } else { "0" }
            ));
        }
        if let Some(pivot_tables) = protection.pivot_tables {
            out.push_str(&format!(
                " pivotTables=\"{}\"",
                if pivot_tables { "1" } else { "0" }
            ));
        }
        if let Some(select_unlocked_cells) = protection.select_unlocked_cells {
            out.push_str(&format!(
                " selectUnlockedCells=\"{}\"",
                if select_unlocked_cells { "1" } else { "0" }
            ));
        }
        out.push_str("/>");
    }
    if !sheet.hyperlinks.is_empty() {
        out.push_str("<hyperlinks>");
        for hl in &sheet.hyperlinks {
            out.push_str(&format!(
                "<hyperlink ref=\"{}\"",
                encode_xml_text(&hl.reference)
            ));
            if let Some(target) = &hl.target {
                out.push_str(&format!(" location=\"{}\"", encode_xml_text(target)));
            }
            if let Some(text) = &hl.text {
                out.push_str(&format!(" display=\"{}\"", encode_xml_text(text)));
            }
            if let Some(tooltip) = &hl.tooltip {
                out.push_str(&format!(" tooltip=\"{}\"", encode_xml_text(tooltip)));
            }
            out.push_str("/>");
        }
        out.push_str("</hyperlinks>");
    }
    if !sheet.drawings.is_empty() {
        for drawing in &sheet.drawings {
            out.push_str(&format!(
                "<drawing r:id=\"{}\"/>",
                encode_xml_text(&drawing.name)
            ));
        }
    }
    if !sheet.tables.is_empty() {
        out.push_str(&format!("<tableParts count=\"{}\">", sheet.tables.len()));
        for table in &sheet.tables {
            out.push_str(&format!(
                "<tablePart name=\"{}\" displayName=\"{}\"/>",
                encode_xml_text(&table.name),
                encode_xml_text(&table.range)
            ));
        }
        out.push_str("</tableParts>");
    }
    if !sheet.pivots.is_empty() {
        for pivot in &sheet.pivots {
            out.push_str(&format!(
                "<pivotTableDefinition name=\"{}\" ref=\"{}\"/>",
                encode_xml_text(&pivot.name),
                encode_xml_text(&pivot.range)
            ));
        }
    }
    out.push_str("</worksheet>");
    out
}
/// Serializes a worksheet row.
fn serialize_row(row: &FRow) -> String {
    let mut out = String::new();
    out.push_str(&format!("<row r=\"{}\"", row.index));
    if let Some(height) = row.height {
        out.push_str(&format!(" ht=\"{}\" customHeight=\"1\"", height));
    }
    if let Some(style_id) = row.style_id {
        out.push_str(&format!(" s=\"{}\"", style_id));
    }
    out.push('>');
    for cell in &row.cells {
        out.push_str(&serialize_cell(cell));
    }
    out.push_str("</row>");
    out
}
/// Serializes a worksheet cell.
fn serialize_cell(cell: &FCell) -> String {
    let mut out = String::new();
    out.push_str(&format!("<c r=\"{}\"", cell.reference));
    if let Some(style_id) = cell.style_id {
        out.push_str(&format!(" s=\"{}\"", style_id));
    }
    match &cell.value {
        FCellValue::String(v) => {
            out.push_str(" t=\"inlineStr\"><is><t>");
            out.push_str(&encode_xml_text(v));
            out.push_str("</t></is></c>");
        }
        FCellValue::Number(v) => {
            out.push('>');
            if let Some(f) = &cell.formula {
                out.push_str(&format!("<f>{}</f>", encode_xml_text(f)));
            }
            out.push_str(&format!("<v>{}</v></c>", v));
        }
        FCellValue::Bool(v) => {
            out.push_str(" t=\"b\">");
            if let Some(f) = &cell.formula {
                out.push_str(&format!("<f>{}</f>", encode_xml_text(f)));
            }
            out.push_str(&format!("<v>{}</v></c>", if *v { 1 } else { 0 }));
        }
        FCellValue::Empty => {
            if let Some(f) = &cell.formula {
                out.push('>');
                out.push_str(&format!("<f>{}</f></c>", encode_xml_text(f)));
            } else {
                out.push_str("/>");
            }
        }
    }
    out
}
/// Serializes shared strings.
fn serialize_shared_strings(wb: &FWorkbook) -> String {
    let mut strings: Vec<String> = Vec::new();
    for sheet in &wb.sheets {
        for row in &sheet.rows {
            for cell in &row.cells {
                if let FCellValue::String(v) = &cell.value {
                    if !strings.contains(v) {
                        strings.push(v.clone());
                    }
                }
            }
        }
    }
    let mut out = String::new();
    out.push_str(XML_DECL);
    out.push_str(&format!(
        "<sst xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" count=\"{}\" uniqueCount=\"{}\">",
        strings.len(),
        strings.len()
    ));
    for s in strings {
        out.push_str(&format!("<si><t>{}</t></si>", encode_xml_text(&s)));
    }
    out.push_str("</sst>");
    out
}
/// Serializes styles.
fn serialize_styles(styles: &[FCellStyle]) -> String {
    let mut out = String::new();
    out.push_str(XML_DECL);
    out.push_str(
        "<styleSheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">",
    );
    out.push_str("<fonts count=\"1\"><font><sz val=\"11\"/><name val=\"Calibri\"/></font></fonts>");
    out.push_str("<fills count=\"1\"><fill><patternFill patternType=\"none\"/></fill></fills>");
    out.push_str(
        "<borders count=\"1\"><border><left/><right/><top/><bottom/><diagonal/></border></borders>",
    );
    out.push_str("<cellStyleXfs count=\"1\"><xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\"/></cellStyleXfs>");
    out.push_str(&format!("<cellXfs count=\"{}\">", styles.len().max(1)));
    if styles.is_empty() {
        out.push_str("<xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\"/>");
    } else {
        for style in styles {
            out.push_str("<xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\"");
            if style.alignment.is_some() {
                out.push_str(" applyAlignment=\"1\"");
            }
            out.push('>');
            if let Some(alignment) = style.alignment {
                let horizontal = match alignment {
                    FCellAlignment::Left => Some("left"),
                    FCellAlignment::Center => Some("center"),
                    FCellAlignment::Right => Some("right"),
                    FCellAlignment::Justify => Some("justify"),
                    _ => None,
                };
                let vertical = match alignment {
                    FCellAlignment::Top => Some("top"),
                    FCellAlignment::Bottom => Some("bottom"),
                    _ => None,
                };
                if horizontal.is_some() || vertical.is_some() {
                    out.push_str("<alignment");
                    if let Some(h) = horizontal {
                        out.push_str(&format!(" horizontal=\"{}\"", h));
                    }
                    if let Some(v) = vertical {
                        out.push_str(&format!(" vertical=\"{}\"", v));
                    }
                    out.push_str("/>");
                }
            }
            out.push_str("</xf>");
        }
    }
    out.push_str("</cellXfs>");
    out.push_str("</styleSheet>");
    out
}
