use crate::core::docs::{FCell, FRow, FSheet, FWorkbook};
use crate::core::sheet::{
    FCellAlignment, FCellBorder, FCellBorders, FCellFill, FCellFont, FCellFormat, FCellStyle,
    FCellValue,
};
use crate::core::workbook::{FDefinedName, FWorkbookProperties, FWorkbookView};
use crate::{
    FExcelError, FPackagePart, collect_elements, decode_xml_text, extract_element_inner,
    extract_first_attr, extract_first_text, flag_present,
};
use std::io::{Cursor, Read};
use zip::read::ZipArchive;
/// Parses a complete Excel workbook from raw bytes.
pub(crate) fn parse_workbook(data: &[u8]) -> crate::FExcelResult<FWorkbook> {
    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)?;
    let mut wb = FWorkbook::new_empty();
    let mut parts: Vec<FPackagePart> = Vec::new();
    let mut sheet_paths: Vec<(String, String, String)> = Vec::new();
    let mut shared_strings: Vec<String> = Vec::new();
    let mut styles_xml: Option<Vec<u8>> = None;
    let mut workbook_xml: Option<Vec<u8>> = None;
    let mut theme_xml: Option<Vec<u8>> = None;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().unwrap_or_default().to_string();
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        match name.as_str() {
            "xl/workbook.xml" => workbook_xml = Some(buf),
            "xl/sharedStrings.xml" => shared_strings = parse_shared_strings(&buf)?,
            "xl/styles.xml" => styles_xml = Some(buf),
            "xl/theme/theme1.xml" => theme_xml = Some(buf),
            "xl/_rels/workbook.xml.rels" => {
                sheet_paths = parse_workbook_rels(&buf)?;
            }
            _ => {
                if name.starts_with("xl/worksheets/") && name.ends_with(".xml") {
                    parts.push(FPackagePart {
                        path: name.clone(),
                        data: buf,
                    });
                } else {
                    parts.push(FPackagePart {
                        path: name,
                        data: buf,
                    });
                }
            }
        }
    }
    if let Some(xml) = workbook_xml {
        let text = std::str::from_utf8(&xml)
            .map_err(|e| FExcelError::Xml(format!("workbook.xml is not valid utf-8: {}", e)))?;
        wb.properties = parse_workbook_properties(text);
        wb.view = parse_workbook_view(text);
        wb.defined_names = parse_defined_names(text);
        wb.sheets = parse_sheets(text, &sheet_paths, &parts, &shared_strings)?;
    }
    if let Some(xml) = styles_xml {
        wb.styles = parse_styles(&xml)?;
    }
    if let Some(xml) = theme_xml {
        wb.theme = Some(String::from_utf8_lossy(&xml).to_string());
    }
    wb.package.parts = parts;
    wb.shared_strings = shared_strings;
    wb.dirty = false;
    Ok(wb)
}
/// Parses workbook relationships.
fn parse_workbook_rels(xml: &[u8]) -> crate::FExcelResult<Vec<(String, String, String)>> {
    let text = std::str::from_utf8(xml)
        .map_err(|e| FExcelError::Xml(format!("workbook.xml.rels is not valid utf-8: {}", e)))?;
    let mut rels = Vec::new();
    for rel in collect_elements(text, "Relationship") {
        let id = extract_first_attr(&rel, "Relationship", "Id").unwrap_or_default();
        let target = extract_first_attr(&rel, "Relationship", "Target").unwrap_or_default();
        let mode = extract_first_attr(&rel, "Relationship", "TargetMode");
        rels.push((id, target, mode.unwrap_or_default()));
    }
    Ok(rels)
}
/// Parses shared strings.
fn parse_shared_strings(xml: &[u8]) -> crate::FExcelResult<Vec<String>> {
    let text = std::str::from_utf8(xml)
        .map_err(|e| FExcelError::Xml(format!("sharedStrings.xml is not valid utf-8: {}", e)))?;
    let mut strings = Vec::new();
    for si in collect_elements(text, "si") {
        let mut value = String::new();
        if let Some(t) = extract_first_text(&si, "t") {
            value.push_str(&decode_xml_text(&t));
        } else {
            for r in collect_elements(&si, "r") {
                if let Some(t) = extract_first_text(&r, "t") {
                    value.push_str(&decode_xml_text(&t));
                }
            }
        }
        strings.push(value);
    }
    Ok(strings)
}
/// Parses workbook-level properties.
fn parse_workbook_properties(xml: &str) -> FWorkbookProperties {
    let mut props = FWorkbookProperties::default();
    if let Some(pr) = extract_element_inner(xml, "workbookPr") {
        props.code_name = extract_first_attr(&pr, "workbookPr", "codeName");
        if let Some(v) = extract_first_attr(&pr, "workbookPr", "date1904") {
            props.date1904 = Some(v == "1" || v == "true");
        }
        props.raw_xml = Some(pr);
    }
    props
}
/// Parses workbook view settings.
fn parse_workbook_view(xml: &str) -> FWorkbookView {
    let mut view = FWorkbookView::default();
    if let Some(v) = extract_element_inner(xml, "workbookView") {
        view.active_tab =
            extract_first_attr(&v, "workbookView", "activeTab").and_then(|x| x.parse().ok());
        if let Some(x) = flag_present(&v, "showHorizontalScroll") {
            view.show_horizontal_scroll = Some(x);
        }
        if let Some(x) = flag_present(&v, "showVerticalScroll") {
            view.show_vertical_scroll = Some(x);
        }
        if let Some(x) = flag_present(&v, "showSheetTabs") {
            view.show_sheet_tabs = Some(x);
        }
        view.raw_xml = Some(v);
    }
    view
}
/// Parses defined names.
fn parse_defined_names(xml: &str) -> Vec<FDefinedName> {
    let mut names = Vec::new();
    if let Some(inner) = extract_element_inner(xml, "definedNames") {
        for dn in collect_elements(&inner, "definedName") {
            let name = extract_first_attr(&dn, "definedName", "name").unwrap_or_default();
            let local_sheet_id =
                extract_first_attr(&dn, "definedName", "localSheetId").and_then(|v| v.parse().ok());
            let text = extract_first_text(&dn, "definedName").unwrap_or_default();
            names.push(FDefinedName {
                name,
                text,
                local_sheet_id,
                raw_xml: Some(dn),
            });
        }
    }
    names
}
/// Parses sheets.
fn parse_sheets(
    xml: &str,
    rels: &[(String, String, String)],
    parts: &[FPackagePart],
    shared_strings: &[String],
) -> crate::FExcelResult<Vec<FSheet>> {
    let mut sheets = Vec::new();
    if let Some(sheets_xml) = extract_element_inner(xml, "sheets") {
        for sheet_xml in collect_elements(&sheets_xml, "sheet") {
            let name = extract_first_attr(&sheet_xml, "sheet", "name").unwrap_or_default();
            let rid = extract_first_attr(&sheet_xml, "sheet", "r:id").unwrap_or_default();
            let target = rels
                .iter()
                .find(|(id, _, _)| id == &rid)
                .map(|(_, target, _)| target.clone())
                .unwrap_or_default();
            let path = normalize_sheet_path(&target);
            let data = parts
                .iter()
                .find(|p| p.path == path)
                .map(|p| p.data.clone())
                .unwrap_or_default();
            let mut sheet = FSheet {
                name,
                relationship_id: rid,
                path: path.clone(),
                ..Default::default()
            };
            if !data.is_empty() {
                let text = std::str::from_utf8(&data)
                    .map_err(|e| FExcelError::Xml(format!("{} is not valid utf-8: {}", path, e)))?;
                sheet = parse_sheet(text, sheet, shared_strings)?;
            }
            sheets.push(sheet);
        }
    }
    Ok(sheets)
}
/// Normalizes a relationship target into an archive path.
fn normalize_sheet_path(target: &str) -> String {
    if target.starts_with("/xl/") {
        target.trim_start_matches('/').to_string()
    } else if target.starts_with("xl/") {
        target.to_string()
    } else {
        format!("xl/{}", target)
    }
}
/// Parses a worksheet.
fn parse_sheet(
    xml: &str,
    mut sheet: FSheet,
    shared_strings: &[String],
) -> crate::FExcelResult<FSheet> {
    sheet.raw_xml = Some(xml.to_string());
    if let Some(cols_xml) = extract_element_inner(xml, "cols") {
        for col in collect_elements(&cols_xml, "col") {
            let min = extract_first_attr(&col, "col", "min")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(1);
            let width = extract_first_attr(&col, "col", "width")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(8.43);
            sheet.column_widths.push((min, width));
        }
    }
    if let Some(sheet_data) = extract_element_inner(xml, "sheetData") {
        for row_xml in collect_elements(&sheet_data, "row") {
            let mut row = FRow {
                index: extract_first_attr(&row_xml, "row", "r")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0),
                ..Default::default()
            };
            if let Some(ht) = extract_first_attr(&row_xml, "row", "ht") {
                row.height = ht.parse().ok();
            }
            row.style_id =
                extract_first_attr(&row_xml, "row", "s").and_then(|v| v.parse::<u32>().ok());
            for cell_xml in collect_elements(&row_xml, "c") {
                row.cells.push(parse_cell(&cell_xml, shared_strings));
            }
            row.raw_xml = Some(row_xml.clone());
            sheet.rows.push(row);
        }
    }
    if let Some(merge_xml) = extract_element_inner(xml, "mergeCells") {
        for mc in collect_elements(&merge_xml, "mergeCell") {
            if let Some(r) = extract_first_attr(&mc, "mergeCell", "ref") {
                sheet.merged_cells.push(r);
            }
        }
    }
    if let Some(pane) = extract_element_inner(xml, "pane") {
        sheet.freeze = Some(crate::core::freeze::FFreezePane {
            top_left_cell: extract_first_attr(&pane, "pane", "topLeftCell"),
            x_split: extract_first_attr(&pane, "pane", "xSplit")
                .and_then(|v| v.parse::<u32>().ok()),
            y_split: extract_first_attr(&pane, "pane", "ySplit")
                .and_then(|v| v.parse::<u32>().ok()),
            active_pane: extract_first_attr(&pane, "pane", "activePane"),
            raw_xml: Some(pane),
        });
    }
    if let Some(af) = extract_element_inner(xml, "autoFilter") {
        let mut filter = crate::core::autofilter::FAutoFilter {
            range: extract_first_attr(&af, "autoFilter", "ref").unwrap_or_default(),
            ..Default::default()
        };
        if let Some(fcs) = extract_element_inner(&af, "filterColumns") {
            for fc in collect_elements(&fcs, "filterColumn") {
                let col_id = extract_first_attr(&fc, "filterColumn", "colId")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0);
                let mut values = Vec::new();
                if let Some(filters) = extract_element_inner(&fc, "filters") {
                    for f in collect_elements(&filters, "filter") {
                        if let Some(val) = extract_first_attr(&f, "filter", "val") {
                            values.push(val);
                        }
                    }
                }
                filter
                    .columns
                    .push(crate::core::autofilter::FAutoFilterColumn {
                        col_id,
                        filters: values,
                        raw_xml: Some(fc),
                    });
            }
        }
        filter.raw_xml = Some(af);
        sheet.autofilter = Some(filter);
    }
    if let Some(dvs) = extract_element_inner(xml, "dataValidations") {
        for dv in collect_elements(&dvs, "dataValidation") {
            sheet
                .data_validations
                .push(crate::core::datavalidation::FDataValidation {
                    range: extract_first_attr(&dv, "dataValidation", "sqref").unwrap_or_default(),
                    validation_type: extract_first_attr(&dv, "dataValidation", "type"),
                    operator: extract_first_attr(&dv, "dataValidation", "operator"),
                    formula1: extract_first_text(&dv, "formula1"),
                    formula2: extract_first_text(&dv, "formula2"),
                    allow_blank: extract_first_attr(&dv, "dataValidation", "allowBlank")
                        .map(|v| v == "1" || v == "true"),
                    show_error_message: extract_first_attr(
                        &dv,
                        "dataValidation",
                        "showErrorMessage",
                    )
                    .map(|v| v == "1" || v == "true"),
                    show_input_message: extract_first_attr(
                        &dv,
                        "dataValidation",
                        "showInputMessage",
                    )
                    .map(|v| v == "1" || v == "true"),
                    raw_xml: Some(dv),
                });
        }
    }
    if let Some(cfs) = extract_element_inner(xml, "conditionalFormatting") {
        for cf in collect_elements(&cfs, "cfRule") {
            sheet
                .conditional_formats
                .push(crate::core::conditional::FConditionalFormat {
                    range: extract_first_attr(&cfs, "conditionalFormatting", "sqref")
                        .unwrap_or_default(),
                    format_type: extract_first_attr(&cf, "cfRule", "type"),
                    priority: extract_first_attr(&cf, "cfRule", "priority")
                        .and_then(|v| v.parse::<u32>().ok()),
                    operator: extract_first_attr(&cf, "cfRule", "operator"),
                    formula: extract_first_text(&cf, "formula"),
                    dxf_id: extract_first_attr(&cf, "cfRule", "dxfId")
                        .and_then(|v| v.parse::<u32>().ok()),
                    raw_xml: Some(cf),
                });
        }
    }
    if let Some(ps) = extract_element_inner(xml, "pageSetup") {
        sheet.print = Some(crate::core::print::FPrintSettings {
            paper_size: extract_first_attr(&ps, "pageSetup", "paperSize")
                .and_then(|v| v.parse::<u32>().ok()),
            orientation: extract_first_attr(&ps, "pageSetup", "orientation"),
            fit_to_width: extract_first_attr(&ps, "pageSetup", "fitToWidth")
                .and_then(|v| v.parse::<u32>().ok()),
            fit_to_height: extract_first_attr(&ps, "pageSetup", "fitToHeight")
                .and_then(|v| v.parse::<u32>().ok()),
            print_area: None,
            print_titles: None,
            raw_xml: Some(ps),
        });
    }
    if let Some(sp) = extract_element_inner(xml, "sheetProtection") {
        sheet.protection = Some(crate::core::protection::FSheetProtection {
            password: extract_first_attr(&sp, "sheetProtection", "password"),
            sheet: extract_first_attr(&sp, "sheetProtection", "sheet")
                .map(|v| v == "1" || v == "true"),
            objects: extract_first_attr(&sp, "sheetProtection", "objects")
                .map(|v| v == "1" || v == "true"),
            scenarios: extract_first_attr(&sp, "sheetProtection", "scenarios")
                .map(|v| v == "1" || v == "true"),
            format_cells: extract_first_attr(&sp, "sheetProtection", "formatCells")
                .map(|v| v == "1" || v == "true"),
            format_columns: extract_first_attr(&sp, "sheetProtection", "formatColumns")
                .map(|v| v == "1" || v == "true"),
            format_rows: extract_first_attr(&sp, "sheetProtection", "formatRows")
                .map(|v| v == "1" || v == "true"),
            insert_columns: extract_first_attr(&sp, "sheetProtection", "insertColumns")
                .map(|v| v == "1" || v == "true"),
            insert_rows: extract_first_attr(&sp, "sheetProtection", "insertRows")
                .map(|v| v == "1" || v == "true"),
            insert_hyperlinks: extract_first_attr(&sp, "sheetProtection", "insertHyperlinks")
                .map(|v| v == "1" || v == "true"),
            delete_columns: extract_first_attr(&sp, "sheetProtection", "deleteColumns")
                .map(|v| v == "1" || v == "true"),
            delete_rows: extract_first_attr(&sp, "sheetProtection", "deleteRows")
                .map(|v| v == "1" || v == "true"),
            select_locked_cells: extract_first_attr(&sp, "sheetProtection", "selectLockedCells")
                .map(|v| v == "1" || v == "true"),
            sort: extract_first_attr(&sp, "sheetProtection", "sort")
                .map(|v| v == "1" || v == "true"),
            auto_filter: extract_first_attr(&sp, "sheetProtection", "autoFilter")
                .map(|v| v == "1" || v == "true"),
            pivot_tables: extract_first_attr(&sp, "sheetProtection", "pivotTables")
                .map(|v| v == "1" || v == "true"),
            select_unlocked_cells: extract_first_attr(
                &sp,
                "sheetProtection",
                "selectUnlockedCells",
            )
            .map(|v| v == "1" || v == "true"),
            raw_xml: Some(sp),
        });
    }
    if let Some(hls) = extract_element_inner(xml, "hyperlinks") {
        for hl in collect_elements(&hls, "hyperlink") {
            sheet.hyperlinks.push(crate::core::hyperlink::FHyperlink {
                reference: extract_first_attr(&hl, "hyperlink", "ref").unwrap_or_default(),
                target: extract_first_attr(&hl, "hyperlink", "location"),
                text: extract_first_attr(&hl, "hyperlink", "display"),
                tooltip: extract_first_attr(&hl, "hyperlink", "tooltip"),
                raw_xml: Some(hl),
            });
        }
    }
    if let Some(drawing) = extract_element_inner(xml, "drawing") {
        sheet.drawings.push(crate::core::drawing::FDrawing {
            name: extract_first_attr(&drawing, "drawing", "name").unwrap_or_default(),
            drawing_type: None,
            anchor: None,
            raw_xml: Some(drawing),
        });
    }
    if let Some(table_parts) = extract_element_inner(xml, "tableParts") {
        for tp in collect_elements(&table_parts, "tablePart") {
            sheet.tables.push(crate::core::table::FTable {
                name: extract_first_attr(&tp, "tablePart", "name").unwrap_or_default(),
                display_name: None,
                range: extract_first_attr(&tp, "tablePart", "displayName").unwrap_or_default(),
                columns: Vec::new(),
                style_name: None,
                raw_xml: Some(tp),
            });
        }
    }
    if let Some(pivot) = extract_element_inner(xml, "pivotTableDefinition") {
        sheet.pivots.push(crate::core::pivot::FPivotTable {
            name: extract_first_attr(&pivot, "pivotTableDefinition", "name").unwrap_or_default(),
            range: extract_first_attr(&pivot, "pivotTableDefinition", "ref").unwrap_or_default(),
            fields: Vec::new(),
            raw_xml: Some(pivot),
        });
    }
    Ok(sheet)
}
/// Parses a cell.
fn parse_cell(xml: &str, shared_strings: &[String]) -> FCell {
    let mut cell = FCell {
        reference: extract_first_attr(xml, "c", "r").unwrap_or_default(),
        style_id: extract_first_attr(xml, "c", "s").and_then(|v| v.parse::<u32>().ok()),
        ..Default::default()
    };
    let cell_type = extract_first_attr(xml, "c", "t").unwrap_or_default();
    if let Some(f) = extract_first_text(xml, "f") {
        cell.formula = Some(decode_xml_text(&f));
    }
    if let Some(v) = extract_first_text(xml, "v") {
        let decoded = decode_xml_text(&v);
        cell.value = match cell_type.as_str() {
            "s" => {
                let idx = decoded.parse::<usize>().unwrap_or(0);
                shared_strings
                    .get(idx)
                    .cloned()
                    .map(FCellValue::String)
                    .unwrap_or(FCellValue::String(String::new()))
            }
            "b" => FCellValue::Bool(decoded == "1" || decoded == "true"),
            "str" => FCellValue::String(decoded),
            _ => decoded
                .parse::<f64>()
                .map(FCellValue::Number)
                .unwrap_or(FCellValue::String(decoded)),
        };
    } else if let Some(is) = extract_element_inner(xml, "is") {
        if let Some(t) = extract_first_text(&is, "t") {
            cell.value = FCellValue::String(decode_xml_text(&t));
        }
    }
    cell.raw_xml = Some(xml.to_string());
    cell
}
/// Parses styles.
fn parse_styles(xml: &[u8]) -> crate::FExcelResult<Vec<FCellStyle>> {
    let text = std::str::from_utf8(xml)
        .map_err(|e| FExcelError::Xml(format!("styles.xml is not valid utf-8: {}", e)))?;
    let mut styles = Vec::new();
    if let Some(cell_xfs) = extract_element_inner(text, "cellXfs") {
        for xf in collect_elements(&cell_xfs, "xf") {
            let mut style = FCellStyle::default();
            if let Some(fmt_id) = extract_first_attr(&xf, "xf", "numFmtId") {
                style.format = Some(FCellFormat {
                    num_fmt_id: fmt_id.parse().ok(),
                    format_code: None,
                });
            }
            if let Some(font_id) = extract_first_attr(&xf, "xf", "fontId") {
                if let Some(fonts) = extract_element_inner(text, "fonts") {
                    if let Some(font_xml) =
                        collect_elements(&fonts, "font").get(font_id.parse::<usize>().unwrap_or(0))
                    {
                        let mut font = FCellFont::default();
                        font.name = extract_first_text(font_xml, "name");
                        font.size =
                            extract_first_text(font_xml, "sz").and_then(|v| v.parse::<f64>().ok());
                        font.bold = flag_present(font_xml, "b");
                        font.italic = flag_present(font_xml, "i");
                        font.underline = flag_present(font_xml, "u");
                        font.color = extract_first_attr(font_xml, "color", "rgb");
                        style.font = Some(font);
                    }
                }
            }
            if let Some(fill_id) = extract_first_attr(&xf, "xf", "fillId") {
                if let Some(fills) = extract_element_inner(text, "fills") {
                    if let Some(fill_xml) =
                        collect_elements(&fills, "fill").get(fill_id.parse::<usize>().unwrap_or(0))
                    {
                        let mut fill = FCellFill::default();
                        if let Some(pattern) = extract_element_inner(fill_xml, "patternFill") {
                            fill.pattern =
                                extract_first_attr(&pattern, "patternFill", "patternType");
                            fill.fg_color = extract_first_attr(&pattern, "fgColor", "rgb");
                            fill.bg_color = extract_first_attr(&pattern, "bgColor", "rgb");
                        }
                        style.fill = Some(fill);
                    }
                }
            }
            if let Some(border_id) = extract_first_attr(&xf, "xf", "borderId") {
                if let Some(borders_xml) = extract_element_inner(text, "borders") {
                    if let Some(border_xml) = collect_elements(&borders_xml, "border")
                        .get(border_id.parse::<usize>().unwrap_or(0))
                    {
                        let mut borders = FCellBorders::default();
                        for (tag, slot) in [
                            ("top", &mut borders.top),
                            ("left", &mut borders.left),
                            ("bottom", &mut borders.bottom),
                            ("right", &mut borders.right),
                        ] {
                            if let Some(side) = extract_element_inner(&border_xml, tag) {
                                let mut b = FCellBorder::default();
                                b.style = extract_first_attr(&side, tag, "style");
                                b.color = extract_first_attr(&side, "color", "rgb");
                                *slot = Some(b);
                            }
                        }
                        style.borders = Some(borders);
                    }
                }
            }
            if let Some(alignment) = extract_element_inner(&xf, "alignment") {
                let horizontal =
                    extract_first_attr(&alignment, "alignment", "horizontal").unwrap_or_default();
                let vertical =
                    extract_first_attr(&alignment, "alignment", "vertical").unwrap_or_default();
                style.alignment = Some(match (horizontal.as_str(), vertical.as_str()) {
                    ("center", _) => FCellAlignment::Center,
                    ("right", _) => FCellAlignment::Right,
                    ("justify", _) => FCellAlignment::Justify,
                    (_, "top") => FCellAlignment::Top,
                    (_, "bottom") => FCellAlignment::Bottom,
                    _ => FCellAlignment::General,
                });
            }
            style.raw_xml = Some(xf.clone());
            styles.push(style);
        }
    }
    Ok(styles)
}
