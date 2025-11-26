// CORRECTED STRUCTURE for TEI XML parsing

use crate::tei_data::*;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

fn normalize_whitespace(s: &str) -> String {
    // Collapse runs of whitespace (spaces, tabs, newlines) into a single space,
    // but preserve a single leading and/or trailing space if the original text
    // had leading/trailing whitespace. This prevents loss of separators between
    // inline nodes while still normalizing internal runs.
    let has_leading = s.chars().next().map(|c| c.is_whitespace()).unwrap_or(false);
    let has_trailing = s
        .chars()
        .rev()
        .next()
        .map(|c| c.is_whitespace())
        .unwrap_or(false);
    let inner = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if inner.is_empty() {
        // If the original contained only whitespace, preserve a single space.
        if has_leading || has_trailing {
            return " ".to_string();
        } else {
            return String::new();
        }
    }
    let mut res = String::new();
    if has_leading {
        res.push(' ');
    }
    res.push_str(&inner);
    if has_trailing {
        res.push(' ');
    }
    res
}

pub fn parse_tei_xml(xml_content: &str) -> Result<TeiDocument, String> {
    let mut reader = Reader::from_str(xml_content);
    // Let the parser deliver raw text nodes; normalize whitespace explicitly.
    reader.trim_text(false);

    let mut doc = TeiDocument::new();
    let mut buf = Vec::new();

    let mut temp_metadata = Metadata::default();
    let mut temp_facsimile = Facsimile::default();
    let mut zones = HashMap::new();
    let mut lines = Vec::new();
    let mut footnotes = Vec::new();

    let mut current_line: Option<Line> = None;
    let mut text_buffer: Vec<String> = Vec::new();
    let mut in_body = false;
    let mut in_facsimile = false;
    let mut in_notes_div = false;

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
                    "back" => {
                        // TEI <back> section can contain footnotes/notes
                        in_body = false;
                        in_facsimile = false;
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
                        current_line = Some(Line {
                            facs,
                            content: Vec::new(),
                        });
                        text_buffer.clear();
                    }
                    "ab" if in_body && current_line.is_some() && !in_notes_div => {
                        // Parse inline content for <ab>
                        let ab_nodes = parse_inline_nodes(&mut reader, &mut buf, "ab");
                        if let Some(line) = current_line.as_mut() {
                            line.content.extend(ab_nodes);
                        }
                    }
                    "div" => {
                        // Check if this is a notes div (accept both "notes" and "note")
                        // This can occur in <body> or <back>
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "type" && (value == "notes" || value == "note") {
                                in_notes_div = true;
                                break;
                            }
                        }
                    }
                    "note" if in_notes_div => {
                        // Parse a note in the notes div
                        let mut note_id = String::new();
                        let mut n = String::new();
                        let mut note_counter = footnotes.len() + 1; // Auto-number if n not provided
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            match key.as_str() {
                                "xml:id" | "id" => note_id = value,
                                "n" => n = value,
                                _ => {}
                            }
                        }

                        // If n is not provided, auto-generate from counter
                        if n.is_empty() {
                            n = note_counter.to_string();
                        }

                        // Parse note content
                        let mut content = String::new();
                        let mut note_buf = Vec::new();
                        let mut depth = 1;
                        loop {
                            match reader.read_event_into(&mut note_buf) {
                                Ok(Event::Start(ref ne)) => {
                                    let nname = String::from_utf8_lossy(ne.local_name().as_ref())
                                        .to_string();
                                    if nname == "note" {
                                        depth += 1;
                                    }
                                }
                                Ok(Event::Text(ce)) => {
                                    content.push_str(&ce.unescape().unwrap_or_default());
                                }
                                Ok(Event::End(ref ce)) => {
                                    let cname = String::from_utf8_lossy(ce.local_name().as_ref())
                                        .to_string();
                                    if cname == "note" {
                                        depth -= 1;
                                        if depth == 0 {
                                            break;
                                        }
                                    }
                                }
                                Ok(Event::Eof) => break,
                                _ => {}
                            }
                            note_buf.clear();
                        }

                        footnotes.push(Footnote {
                            id: note_id,
                            n,
                            content,
                        });
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
                    "div" => {
                        if in_notes_div {
                            in_notes_div = false;
                        }
                    }
                    "body" => {
                        if let Some(line) = current_line.take() {
                            lines.push(line);
                        }
                        in_body = false;
                        in_notes_div = false;
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
                let raw = e.unescape().unwrap_or_default().to_string();
                let text = normalize_whitespace(&raw);
                if !text.is_empty() {
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
    doc.footnotes = footnotes;

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
                        let mut abbr = String::new();
                        let mut expan = String::new();
                        let mut sic = String::new();
                        let mut corr = String::new();
                        let mut orig = String::new();
                        let mut reg = String::new();
                        let mut choice_buf = Vec::new();
                        let mut in_abbr = false;
                        let mut in_expan = false;
                        let mut in_sic = false;
                        let mut in_corr = false;
                        let mut in_orig = false;
                        let mut in_reg = false;

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
                                        "orig" => in_orig = true,
                                        "reg" => in_reg = true,
                                        _ => {}
                                    }
                                }
                                Ok(Event::Text(ref t)) => {
                                    let text = t.unescape().unwrap_or_default().to_string();
                                    if in_sic {
                                        sic.push_str(&text);
                                    } else if in_corr {
                                        corr.push_str(&text);
                                    } else if in_abbr {
                                        abbr.push_str(&text);
                                    } else if in_expan {
                                        expan.push_str(&text);
                                    } else if in_orig {
                                        orig.push_str(&text);
                                    } else if in_reg {
                                        reg.push_str(&text);
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
                                        "orig" => in_orig = false,
                                        "reg" => in_reg = false,
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
                        } else if !orig.is_empty() || !reg.is_empty() {
                            nodes.push(TextNode::Regularised { orig, reg });
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
                        let mut tipo = String::new();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "value" {
                                value = val.parse().unwrap_or(0);
                            } else if key == "type" {
                                tipo = val;
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
                            tipo,
                            text: num_text,
                        });
                    }
                    "persName" => {
                        let mut tipo = String::new();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "type" {
                                tipo = val;
                            }
                        }
                        let mut name = String::new();
                        let mut pers_buf = Vec::new();
                        loop {
                            match reader.read_event_into(&mut pers_buf) {
                                Ok(Event::Text(ce)) => {
                                    name.push_str(&ce.unescape().unwrap_or_default());
                                }
                                Ok(Event::End(ref ce)) => {
                                    let cname = String::from_utf8_lossy(ce.local_name().as_ref())
                                        .to_string();
                                    if cname == "persName" {
                                        break;
                                    }
                                }
                                Ok(Event::Eof) => break,
                                _ => {}
                            }
                            pers_buf.clear();
                        }
                        nodes.push(TextNode::PersName { name, tipo });
                    }
                    "placeName" => {
                        let mut name = String::new();
                        let mut place_buf = Vec::new();
                        loop {
                            match reader.read_event_into(&mut place_buf) {
                                Ok(Event::Text(ce)) => {
                                    name.push_str(&ce.unescape().unwrap_or_default());
                                }
                                Ok(Event::End(ref ce)) => {
                                    let cname = String::from_utf8_lossy(ce.local_name().as_ref())
                                        .to_string();
                                    if cname == "placeName" {
                                        break;
                                    }
                                }
                                Ok(Event::Eof) => break,
                                _ => {}
                            }
                            place_buf.clear();
                        }
                        nodes.push(TextNode::PlaceName { name });
                    }
                    "rs" => {
                        let mut rs_type = String::new();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "type" {
                                rs_type = val;
                            }
                        }
                        let mut content = String::new();
                        let mut rs_buf = Vec::new();
                        loop {
                            match reader.read_event_into(&mut rs_buf) {
                                Ok(Event::Text(ce)) => {
                                    content.push_str(&ce.unescape().unwrap_or_default());
                                }
                                Ok(Event::End(ref ce)) => {
                                    let cname = String::from_utf8_lossy(ce.local_name().as_ref())
                                        .to_string();
                                    if cname == "rs" {
                                        break;
                                    }
                                }
                                Ok(Event::Eof) => break,
                                _ => {}
                            }
                            rs_buf.clear();
                        }
                        nodes.push(TextNode::RsType { rs_type, content });
                    }
                    "note" => {
                        // Could be inline note or note reference (with target attribute)
                        let mut n = String::new();
                        let mut target = String::new();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            match key.as_str() {
                                "n" => n = val,
                                "target" => target = val,
                                _ => {}
                            }
                        }

                        // If has target, it's a note reference
                        if !target.is_empty() {
                            let mut content = String::new();
                            let mut note_buf = Vec::new();
                            loop {
                                match reader.read_event_into(&mut note_buf) {
                                    Ok(Event::Text(ce)) => {
                                        content.push_str(&ce.unescape().unwrap_or_default());
                                    }
                                    Ok(Event::End(ref ce)) => {
                                        let cname =
                                            String::from_utf8_lossy(ce.local_name().as_ref())
                                                .to_string();
                                        if cname == "note" {
                                            break;
                                        }
                                    }
                                    Ok(Event::Eof) => break,
                                    _ => {}
                                }
                                note_buf.clear();
                            }
                            let note_id = target.trim_start_matches('#').to_string();
                            let display_n = if !content.is_empty() { content } else { n };
                            nodes.push(TextNode::NoteRef {
                                note_id,
                                n: display_n,
                            });
                        } else {
                            // Inline note
                            let mut content = String::new();
                            let mut note_buf = Vec::new();
                            loop {
                                match reader.read_event_into(&mut note_buf) {
                                    Ok(Event::Text(ce)) => {
                                        content.push_str(&ce.unescape().unwrap_or_default());
                                    }
                                    Ok(Event::End(ref ce)) => {
                                        let cname =
                                            String::from_utf8_lossy(ce.local_name().as_ref())
                                                .to_string();
                                        if cname == "note" {
                                            break;
                                        }
                                    }
                                    Ok(Event::Eof) => break,
                                    _ => {}
                                }
                                note_buf.clear();
                            }
                            nodes.push(TextNode::InlineNote { content, n });
                        }
                    }
                    "ref" => {
                        let mut ref_type = String::new();
                        let mut target = String::new();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "type" {
                                ref_type = val;
                            } else if key == "target" {
                                target = val;
                            }
                        }
                        let mut content = String::new();
                        let mut ref_buf = Vec::new();
                        loop {
                            match reader.read_event_into(&mut ref_buf) {
                                Ok(Event::Text(ce)) => {
                                    content.push_str(&ce.unescape().unwrap_or_default());
                                }
                                Ok(Event::End(ref ce)) => {
                                    let cname = String::from_utf8_lossy(ce.local_name().as_ref())
                                        .to_string();
                                    if cname == "ref" {
                                        break;
                                    }
                                }
                                Ok(Event::Eof) => break,
                                _ => {}
                            }
                            ref_buf.clear();
                        }

                        // Check if this is a note reference
                        if ref_type == "note" && target.starts_with('#') {
                            let note_id = target.trim_start_matches('#').to_string();
                            nodes.push(TextNode::NoteRef {
                                note_id,
                                n: content,
                            });
                        } else {
                            nodes.push(TextNode::Ref {
                                ref_type,
                                target,
                                content,
                            });
                        }
                    }
                    "unclear" => {
                        let mut reason = String::new();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "reason" {
                                reason = val;
                            }
                        }
                        let mut content = String::new();
                        let mut unclear_buf = Vec::new();
                        loop {
                            match reader.read_event_into(&mut unclear_buf) {
                                Ok(Event::Text(ce)) => {
                                    content.push_str(&ce.unescape().unwrap_or_default());
                                }
                                Ok(Event::End(ref ce)) => {
                                    let cname = String::from_utf8_lossy(ce.local_name().as_ref())
                                        .to_string();
                                    if cname == "unclear" {
                                        break;
                                    }
                                }
                                Ok(Event::Eof) => break,
                                _ => {}
                            }
                            unclear_buf.clear();
                        }
                        nodes.push(TextNode::Unclear { reason, content });
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
                let raw = e.unescape().unwrap_or_default().to_string();
                let t = normalize_whitespace(&raw);
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
