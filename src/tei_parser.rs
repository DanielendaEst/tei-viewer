// CORRECTED STRUCTURE for TEI XML parsing

use crate::tei_data::*;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

pub fn parse_tei_xml(xml_content: &str) -> Result<TeiDocument, String> {
    let mut reader = Reader::from_str(xml_content);
    reader.trim_text(true);

    let mut doc = TeiDocument::new();
    let mut buf = Vec::new();

    let mut temp_metadata = Metadata::default();
    let mut temp_facsimile = Facsimile::default();
    let mut zones = HashMap::new();
    let mut lines = Vec::new();

    let mut current_line: Option<Line> = None;
    let mut text_buffer: Vec<String> = Vec::new();
    let mut in_body = false;
    let mut in_facsimile = false;

    // SINGLE, FLAT EVENT LOOP - no nested parsers fighting each other
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                match name.as_str() {
                    // ===== FACSIMILE SECTION =====
                    "facsimile" => {
                        in_facsimile = true;
                    }
                    "surface" => {
                        if in_facsimile {
                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                if key == "xml:id" {
                                    temp_facsimile.surface_id = value;
                                }
                            }
                        }
                    }
                    "graphic" => {
                        if in_facsimile {
                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                match key.as_str() {
                                    "url" => {
                                        temp_facsimile.image_url = value;
                                    }
                                    "width" => {
                                        temp_facsimile.width = value.parse().unwrap_or(0);
                                    }
                                    "height" => {
                                        temp_facsimile.height = value.parse().unwrap_or(0);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    "zone" => {
                        if in_facsimile {
                            let mut zone = Zone {
                                id: String::new(),
                                zone_type: String::new(),
                                points: Vec::new(),
                            };
                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                match key.as_str() {
                                    "xml:id" => zone.id = value,
                                    "type" => zone.zone_type = value,
                                    "points" => zone.points = parse_points_allow_float(&value),
                                    _ => {}
                                }
                            }
                            if !zone.id.is_empty() {
                                let zone_id_clone = zone.id.clone();
                                zones.insert(zone_id_clone.clone(), zone);
                            }
                        }
                    }

                    // ===== BODY/TRANSCRIPTION SECTION =====
                    "body" => {
                        in_body = true;
                        in_facsimile = false; // Exit facsimile mode
                    }
                    "lb" if in_body => {
                        // Save previous line if exists
                        if let Some(line) = current_line.take() {
                            lines.push(line);
                        }

                        // Start new line
                        let mut facs = String::new();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "facs" {
                                facs = value.trim_start_matches('#').to_string();
                            }
                        }
                        let facs_clone = facs.clone();
                        current_line = Some(Line {
                            facs,
                            content: Vec::new(),
                        });
                        text_buffer.clear();
                    }
                    "ab" if in_body && current_line.is_some() => {
                        // Parse inline content for <ab>
                        let ab_nodes = parse_inline_nodes(&mut reader, &mut buf, "ab");
                        if let Some(line) = current_line.as_mut() {
                            line.content.extend(ab_nodes);
                        }
                    }

                    // ===== METADATA SECTION =====
                    "title" => {
                        // Collect text until closing tag
                        text_buffer.clear();
                    }
                    "author" | "editor" | "edition" | "language" | "country" | "settlement"
                    | "institution" | "collection" => {
                        text_buffer.clear();
                    }
                    _ => {}
                }
            }

            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                match name.as_str() {
                    "facsimile" => {
                        in_facsimile = false;
                    }
                    "body" => {
                        if let Some(line) = current_line.take() {
                            lines.push(line);
                        }
                        in_body = false;
                    }
                    "title" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.title = text_buffer.join("");
                        }
                        text_buffer.clear();
                    }
                    "author" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.author = text_buffer.join("");
                        }
                        text_buffer.clear();
                    }
                    "editor" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.editor = text_buffer.join("");
                        }
                        text_buffer.clear();
                    }
                    "edition" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.edition_type = text_buffer.join("");
                        }
                        text_buffer.clear();
                    }
                    "language" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.language = text_buffer.join("");
                        }
                        text_buffer.clear();
                    }
                    "country" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.country = Some(text_buffer.join(""));
                        }
                        text_buffer.clear();
                    }
                    "settlement" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.settlement = Some(text_buffer.join(""));
                        }
                        text_buffer.clear();
                    }
                    "institution" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.institution = Some(text_buffer.join(""));
                        }
                        text_buffer.clear();
                    }
                    "collection" => {
                        if !text_buffer.is_empty() {
                            temp_metadata.collection = Some(text_buffer.join(""));
                        }
                        text_buffer.clear();
                    }
                    _ => {}
                }
            }

            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if !text.trim().is_empty() {
                    text_buffer.push(text);
                }
            }

            Ok(Event::Empty(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                // Handle <graphic /> and <zone /> self-closing tags in facsimile
                if in_facsimile && name == "graphic" {
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "url" => temp_facsimile.image_url = value,
                            "width" => temp_facsimile.width = value.parse().unwrap_or(0),
                            "height" => temp_facsimile.height = value.parse().unwrap_or(0),
                            _ => {}
                        }
                    }
                } else if in_facsimile && name == "zone" {
                    let mut zone = Zone {
                        id: String::new(),
                        zone_type: String::new(),
                        points: Vec::new(),
                    };
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "xml:id" => zone.id = value,
                            "type" => zone.zone_type = value,
                            "points" => zone.points = parse_points_allow_float(&value),
                            _ => {}
                        }
                    }
                    if !zone.id.is_empty() {
                        zones.insert(zone.id.clone(), zone);
                    }
                } else if name == "lb" && in_body {
                    // Self-closing <lb/>
                    if let Some(line) = current_line.take() {
                        lines.push(line);
                    }

                    let mut facs = String::new();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        if key == "facs" {
                            facs = value.trim_start_matches('#').to_string();
                        }
                    }

                    current_line = Some(Line {
                        facs,
                        content: Vec::new(),
                    });
                    text_buffer.clear();
                }
            }

            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(format!(
                    "XML parsing error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => {}
        }
        buf.clear();
    }

    // Validate facsimile was parsed correctly

    temp_facsimile.zones = zones;
    doc.metadata = temp_metadata;
    doc.facsimile = temp_facsimile;
    doc.lines = lines;

    Ok(doc)
}

/// Parse inline nodes within elements like <ab>, <choice>, etc.
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
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "choice" => {
                        // Handle <choice> with <sic>, <corr>, <abbr>, <expan>
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
                                    let cname = String::from_utf8_lossy(ce.local_name().as_ref())
                                        .to_string();
                                    match cname.as_str() {
                                        "sic" => in_sic = true,
                                        "corr" => in_corr = true,
                                        "abbr" => in_abbr = true,
                                        "expan" => in_expan = true,
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
                                Ok(Event::End(ref ce)) => {
                                    let cname = String::from_utf8_lossy(ce.local_name().as_ref())
                                        .to_string();
                                    match cname.as_str() {
                                        "sic" => in_sic = false,
                                        "corr" => in_corr = false,
                                        "abbr" => in_abbr = false,
                                        "expan" => in_expan = false,
                                        "choice" => break,
                                        _ => {}
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
                    "hi" => {
                        let mut rend = String::new();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "rend" {
                                rend = value;
                            }
                        }
                        // Recursively parse nested content
                        let inner = parse_inline_nodes(reader, buf, "hi");
                        let content = inner
                            .into_iter()
                            .filter_map(|n| match n {
                                TextNode::Text { content } => Some(content),
                                _ => None,
                            })
                            .collect::<String>();
                        nodes.push(TextNode::Hi { rend, content });
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
                                    let cname = String::from_utf8_lossy(ce.local_name().as_ref())
                                        .to_string();
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
                    _ => {
                        // Unknown tag: recurse
                        let _ = parse_inline_nodes(reader, buf, &name);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
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

fn parse_points_allow_float(points_str: &str) -> Vec<(u32, u32)> {
    points_str
        .split_whitespace()
        .filter_map(|pair| {
            let coords: Vec<&str> = pair.split(',').collect();
            if coords.len() == 2 {
                let x_parsed = coords[0].trim().parse::<f32>().ok();
                let y_parsed = coords[1].trim().parse::<f32>().ok();
                if let (Some(xf), Some(yf)) = (x_parsed, y_parsed) {
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
