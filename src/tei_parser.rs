/*
// src/tei_parser.rs
//
// Quick-XML based TEI parser (robust parser) - keeps only the proper `quick_xml` parser
// and helper for parsing zone points. The old fragile `parse_tei_simple` (string-based)
// parser has been removed.
*/

use crate::tei_data::*;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

/// Parse a TEI XML document using `quick-xml` and return a populated `TeiDocument`.
/// This parser is namespace-agile in the sense that it compares element local names
/// as strings and focuses on elements relevant to the viewer:
/// - surface / graphic for facsimile image metadata
/// - zone elements for polygonal regions
/// - body / lb for lines and in-line semantic markup
pub fn parse_tei_xml(xml_content: &str) -> Result<TeiDocument, String> {
    let mut reader = Reader::from_str(xml_content);
    reader.trim_text(true);

    let mut doc = TeiDocument::new();
    let mut buf = Vec::new();
    let mut current_path = Vec::new();
    let mut temp_metadata = Metadata::default();
    let mut temp_facsimile = Facsimile::default();
    let mut zones = HashMap::new();
    let mut lines = Vec::new();
    let mut current_line: Option<Line> = None;
    let mut text_buffer = Vec::new();
    let mut in_body = false;

    // --- Inline node parser ---
    fn parse_inline_nodes<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        buf: &mut Vec<u8>,
        break_tag: &str,
    ) -> Vec<TextNode> {
        let mut nodes = Vec::new();
        let mut local_buf = Vec::new();
        loop {
            match reader.read_event_into(&mut local_buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match name.as_str() {
                        "choice" => {
                            let mut sic = String::new();
                            let mut corr = String::new();
                            let mut abbr = String::new();
                            let mut expan = String::new();
                            let mut choice_buf = Vec::new();
                            let mut in_sic = false;
                            let mut in_corr = false;
                            let mut in_abbr = false;
                            let mut in_expan = false;
                            loop {
                                match reader.read_event_into(&mut choice_buf) {
                                    Ok(Event::Start(ref ce)) => {
                                        let cname =
                                            String::from_utf8_lossy(ce.name().as_ref()).to_string();
                                        match cname.as_str() {
                                            "sic" => in_sic = true,
                                            "corr" => in_corr = true,
                                            "abbr" => in_abbr = true,
                                            "expan" => in_expan = true,
                                            _ => {}
                                        }
                                    }
                                    Ok(Event::End(ref ce)) => {
                                        let cname =
                                            String::from_utf8_lossy(ce.name().as_ref()).to_string();
                                        match cname.as_str() {
                                            "sic" => in_sic = false,
                                            "corr" => in_corr = false,
                                            "abbr" => in_abbr = false,
                                            "expan" => in_expan = false,
                                            "choice" => break,
                                            _ => {}
                                        }
                                    }
                                    Ok(Event::Text(ce)) => {
                                        let t = ce.unescape().unwrap_or_default().to_string();
                                        if in_sic {
                                            sic.push_str(&t);
                                        } else if in_corr {
                                            corr.push_str(&t);
                                        } else if in_abbr {
                                            abbr.push_str(&t);
                                        } else if in_expan {
                                            expan.push_str(&t);
                                        }
                                    }
                                    Ok(Event::Eof) => break,
                                    _ => {}
                                }
                                choice_buf.clear();
                            }
                            if !abbr.is_empty() || !expan.is_empty() {
                                nodes.push(TextNode::Abbr { abbr, expan });
                            } else if !sic.is_empty() || !corr.is_empty() {
                                nodes.push(TextNode::Choice { sic, corr });
                            }
                        }
                        "num" => {
                            let mut value = 0;
                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                if key == "value" {
                                    value = val.parse().unwrap_or(0);
                                }
                            }
                            let mut num_text = String::new();
                            let mut num_buf = Vec::new();
                            loop {
                                match reader.read_event_into(&mut num_buf) {
                                    Ok(Event::Text(ce)) => {
                                        num_text.push_str(&ce.unescape().unwrap_or_default());
                                    }
                                    Ok(Event::End(ref ce)) => {
                                        let cname =
                                            String::from_utf8_lossy(ce.name().as_ref()).to_string();
                                        if cname == "num" {
                                            break;
                                        }
                                    }
                                    Ok(Event::Eof) => break,
                                    _ => {}
                                }
                                num_buf.clear();
                            }
                            nodes.push(TextNode::Num {
                                value,
                                text: num_text,
                            });
                        }
                        "hi" => {
                            let mut rend = String::new();
                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                if key == "rend" {
                                    rend = val;
                                }
                            }
                            let mut hi_content = parse_inline_nodes(reader, buf, "hi");
                            let hi_text = hi_content
                                .into_iter()
                                .map(|n| match n {
                                    TextNode::Text { content } => content,
                                    _ => "".to_string(),
                                })
                                .collect::<String>();
                            nodes.push(TextNode::Hi {
                                rend,
                                content: hi_text,
                            });
                        }
                        _ => {
                            // For unknown tags, just parse their children as text
                            let mut unknown_content = parse_inline_nodes(reader, buf, &name);
                            for n in unknown_content {
                                nodes.push(n);
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == break_tag {
                        break;
                    }
                }
                Ok(Event::Text(e)) => {
                    let t = e.unescape().unwrap_or_default().to_string();
                    if !t.is_empty() {
                        nodes.push(TextNode::Text { content: t });
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            local_buf.clear();
        }
        nodes
    }

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                current_path.push(name.clone());

                match name.as_str() {
                    "surface" => {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                match key.as_str() {
                                    "xml:id" => temp_facsimile.surface_id = value,
                                    _ => {}
                                }
                            }
                        }
                    }
                    "graphic" => {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                match key.as_str() {
                                    "url" => temp_facsimile.image_url = value,
                                    "width" => temp_facsimile.width = value.parse().unwrap_or(0),
                                    "height" => temp_facsimile.height = value.parse().unwrap_or(0),
                                    _ => {}
                                }
                            }
                        }
                    }
                    "zone" => {
                        let mut zone = Zone {
                            id: String::new(),
                            zone_type: String::new(),
                            points: Vec::new(),
                        };
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                match key.as_str() {
                                    "xml:id" => zone.id = value,
                                    "type" => zone.zone_type = value,
                                    "points" => zone.points = parse_points_allow_float(&value),
                                    _ => {}
                                }
                            }
                        }
                        if !zone.id.is_empty() {
                            zones.insert(zone.id.clone(), zone);
                        }
                    }
                    "body" => {
                        in_body = true;
                    }
                    "lb" => {
                        if in_body {
                            // Save previous line if exists
                            if let Some(line) = current_line.take() {
                                lines.push(line);
                            }
                            // Start new line
                            let mut facs = String::new();
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    let key =
                                        String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                    let value = String::from_utf8_lossy(&attr.value).to_string();
                                    if key == "facs" {
                                        facs = value.trim_start_matches('#').to_string();
                                    }
                                }
                            }
                            current_line = Some(Line {
                                facs,
                                content: Vec::new(),
                            });
                            text_buffer.clear();
                        }
                    }
                    "ab" => {
                        if in_body {
                            if let Some(line) = current_line.as_mut() {
                                let mut ab_nodes = parse_inline_nodes(&mut reader, &mut buf, "ab");
                                line.content.append(&mut ab_nodes);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if in_body && current_line.is_some() {
                    match name.as_str() {
                        "ab" => {
                            // When closing an <ab>, push any accumulated text as a TextNode::Text
                            if let Some(line) = current_line.as_mut() {
                                if !text_buffer.is_empty() {
                                    line.content.push(TextNode::Text {
                                        content: text_buffer.join(""),
                                    });
                                    text_buffer.clear();
                                }
                            }
                            // Save the line and reset for the next one
                            if let Some(line) = current_line.take() {
                                lines.push(line);
                            }
                        }
                        "p" => {
                            // If you want to handle <p> as a multi-line paragraph, keep this block.
                            // Otherwise, you may want to leave it empty or handle as needed.
                        }
                        "num" => {
                            if let Some(line) = current_line.as_mut() {
                                let content = text_buffer.join("");
                                // Try to extract value from attributes wasn't tracked here; set placeholder 0
                                line.content.push(TextNode::Num {
                                    value: 0,
                                    text: content,
                                });
                            }
                            text_buffer.clear();
                        }
                        "persName" => {
                            if let Some(line) = current_line.as_mut() {
                                let name = text_buffer.join("");
                                line.content.push(TextNode::PersName { name });
                            }
                            text_buffer.clear();
                        }
                        "placeName" => {
                            if let Some(line) = current_line.as_mut() {
                                let name = text_buffer.join("");
                                line.content.push(TextNode::PlaceName { name });
                            }
                            text_buffer.clear();
                        }
                        "rs" => {
                            if let Some(line) = current_line.as_mut() {
                                let content = text_buffer.join("");
                                line.content.push(TextNode::RsType {
                                    rs_type: String::new(),
                                    content,
                                });
                            }
                            text_buffer.clear();
                        }
                        "note" => {
                            if let Some(line) = current_line.as_mut() {
                                let content = text_buffer.join("");
                                line.content.push(TextNode::Note {
                                    content,
                                    note_id: format!("note_{}", lines.len()),
                                });
                            }
                            text_buffer.clear();
                        }
                        "hi" => {
                            if let Some(line) = current_line.as_mut() {
                                let content = text_buffer.join("");
                                line.content.push(TextNode::Hi {
                                    rend: String::new(),
                                    content,
                                });
                            }
                            text_buffer.clear();
                        }
                        _ => {}
                    }
                }

                match name.as_str() {
                    "title" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.title = text_buffer.join("");
                            text_buffer.clear();
                        }
                    }
                    "author" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.author = text_buffer.join("");
                            text_buffer.clear();
                        }
                    }
                    "editor" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.editor = text_buffer.join("");
                            text_buffer.clear();
                        }
                    }
                    "edition" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.edition_type = text_buffer.join("");
                            text_buffer.clear();
                        }
                    }
                    "language" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.language = text_buffer.join("");
                            text_buffer.clear();
                        }
                    }
                    "country" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.country = Some(text_buffer.join(""));
                            text_buffer.clear();
                        }
                    }
                    "settlement" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.settlement = Some(text_buffer.join(""));
                            text_buffer.clear();
                        }
                    }
                    "institution" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.institution = Some(text_buffer.join(""));
                            text_buffer.clear();
                        }
                    }
                    "collection" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.collection = Some(text_buffer.join(""));
                            text_buffer.clear();
                        }
                    }
                    "body" => {
                        in_body = false;
                        // Save last line if exists
                        if let Some(line) = current_line.take() {
                            lines.push(line);
                        }
                    }
                    _ => {}
                }

                if !current_path.is_empty() {
                    current_path.pop();
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if !text.trim().is_empty() {
                    if in_body && current_line.is_some() {
                        text_buffer.push(text.clone());
                    } else {
                        text_buffer.push(text);
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if name == "lb" && in_body {
                    // Save previous line if exists
                    if let Some(line) = current_line.take() {
                        lines.push(line);
                    }
                    // Start new line (empty lb)
                    let mut facs = String::new();
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "facs" {
                                facs = value.trim_start_matches('#').to_string();
                            }
                        }
                    }
                    current_line = Some(Line {
                        facs,
                        content: Vec::new(),
                    });
                    text_buffer.clear();
                } else if name == "zone" {
                    let mut zone = Zone {
                        id: String::new(),
                        zone_type: String::new(),
                        points: Vec::new(),
                    };
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            match key.as_str() {
                                "xml:id" => zone.id = value,
                                "type" => zone.zone_type = value,
                                "points" => zone.points = parse_points_allow_float(&value),
                                _ => {}
                            }
                        }
                    }
                    if !zone.id.is_empty() {
                        zones.insert(zone.id.clone(), zone);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(format!(
                    "Error parsing XML at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => {}
        }
        buf.clear();
    }

    // Process lines to extract text content (finalization)
    for line in &mut lines {
        if line.content.is_empty() && !text_buffer.is_empty() {
            line.content.push(TextNode::Text {
                content: text_buffer.join(""),
            });
        }
    }

    temp_facsimile.zones = zones;
    doc.metadata = temp_metadata;
    doc.facsimile = temp_facsimile;
    doc.lines = lines;

    Ok(doc)
}

/// Parse a TEI `points` string allowing floating point coordinates.
/// Returns a Vec<(u32,u32)> by rounding coordinates to the nearest integer.
/// Example input: "586,455 597.2,452.6 600,460"
fn parse_points_allow_float(points_str: &str) -> Vec<(u32, u32)> {
    points_str
        .split_whitespace()
        .filter_map(|pair| {
            let coords: Vec<&str> = pair.split(',').collect();
            if coords.len() == 2 {
                let x_parsed = coords[0].trim().parse::<f32>().ok();
                let y_parsed = coords[1].trim().parse::<f32>().ok();
                if let (Some(xf), Some(yf)) = (x_parsed, y_parsed) {
                    // Handle NaN / negative gracefully: clamp negatives to zero
                    if xf.is_finite() && yf.is_finite() {
                        let xi = if xf.is_sign_negative() {
                            0
                        } else {
                            xf.round() as u32
                        };
                        let yi = if yf.is_sign_negative() {
                            0
                        } else {
                            yf.round() as u32
                        };
                        return Some((xi, yi));
                    }
                }
            }
            None
        })
        .collect()
}
