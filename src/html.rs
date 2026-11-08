use std::collections::BTreeMap;

use crate::dom::{Document, NodeKind};

/// Parse a forgiving HTML subset into a lightweight DOM; this is not the HTML Living Standard parser.
pub fn parse(source: &str) -> Document {
    let mut doc = Document::new();
    let mut stack = vec![0usize];
    let mut cursor = 0usize;

    while cursor < source.len() {
        let rest = &source[cursor..];
        if rest.starts_with("<!--") {
            cursor += rest.find("-->").map_or(rest.len(), |i| i + 3);
            continue;
        }
        if rest.starts_with('<') {
            if let Some(end) = find_tag_end(rest) {
                let raw = rest[1..end].trim();
                let after_tag = cursor + end + 1;
                if raw.is_empty() || raw.starts_with('!') || raw.starts_with('?') {
                    cursor = after_tag;
                    continue;
                }
                if let Some(closing) = raw.strip_prefix('/') {
                    let name = closing.split_whitespace().next().unwrap_or("").to_ascii_lowercase();
                    if let Some(pos) = stack.iter().rposition(|&id| element_tag(&doc, id) == Some(name.as_str())) {
                        if pos > 0 { stack.truncate(pos); }
                    }
                    cursor = after_tag;
                    continue;
                }
                let self_closing = raw.ends_with('/');
                let raw = raw.strip_suffix('/').unwrap_or(raw).trim_end();
                let (tag, attributes) = parse_start_tag(raw);
                if tag.is_empty() || !tag.as_bytes()[0].is_ascii_alphabetic() {
                    append_text(&mut doc, *stack.last().unwrap_or(&0), "<");
                    cursor += 1;
                    continue;
                }

                close_implied_elements(&doc, &mut stack, &tag);
                let parent = *stack.last().unwrap_or(&0);
                let id = doc.append(parent, NodeKind::Element { tag: tag.clone(), attributes });
                cursor = after_tag;

                if !self_closing && !is_void(&tag) {
                    stack.push(id);
                    if is_raw_text(&tag) {
                        let content_start = cursor;
                        let closing = find_raw_text_close(source, content_start, &tag);
                        let content_end = closing.unwrap_or(source.len());
                        if content_end > content_start {
                            let content = &source[content_start..content_end];
                            let text = if matches!(tag.as_str(), "textarea" | "title") { decode_entities(content) } else { content.to_string() };
                            append_text(&mut doc, id, &text);
                        }
                        cursor = content_end;
                    }
                }
                continue;
            }
            // Treat a malformed less-than sign as text rather than stalling on the same input byte.
            append_text(&mut doc, *stack.last().unwrap_or(&0), "<");
            cursor += 1;
            continue;
        }

        let next = rest.find('<').unwrap_or(rest.len());
        let text = decode_entities(&rest[..next]);
        append_text(&mut doc, *stack.last().unwrap_or(&0), &text);
        cursor += next;
    }
    doc
}

fn find_tag_end(input: &str) -> Option<usize> {
    let mut quote = None;
    for (index, ch) in input.char_indices().skip(1) {
        match (quote, ch) {
            (Some(current), ch) if current == ch => quote = None,
            (None, '\'' | '"') => quote = Some(ch),
            (None, '>') => return Some(index),
            _ => {}
        }
    }
    None
}

fn find_raw_text_close(source: &str, from: usize, tag: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let needle = format!("</{}", tag).into_bytes();
    if from > bytes.len() || needle.len() > bytes.len() { return None; }
    for i in from..=bytes.len().saturating_sub(needle.len()) {
        if bytes[i..i + needle.len()].eq_ignore_ascii_case(&needle) {
            let boundary = bytes.get(i + needle.len()).copied();
            if boundary.is_some_and(|b| b.is_ascii_whitespace() || b == b'>' || b == b'/') {
                return Some(i);
            }
        }
    }
    None
}

fn close_implied_elements(doc: &Document, stack: &mut Vec<usize>, incoming: &str) {
    let should_close = match incoming {
        "li" => Some("li"),
        "dt" | "dd" => Some("dt"),
        "p" | "address" | "article" | "blockquote" | "div" | "dl" | "fieldset" | "footer" | "form" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "header" | "hr" | "main" | "nav" | "ol" | "pre" | "section" | "table" | "ul" => Some("p"),
        _ => None,
    };
    if let Some(tag) = should_close {
        if let Some(pos) = stack.iter().rposition(|&id| element_tag(doc, id) == Some(tag)) {
            if pos > 0 { stack.truncate(pos); }
        }
    }
    if incoming == "dd" {
        if let Some(pos) = stack.iter().rposition(|&id| element_tag(doc, id) == Some("dd")) {
            if pos > 0 { stack.truncate(pos); }
        }
    }
}

fn append_text(doc: &mut Document, parent: usize, text: &str) {
    if !text.is_empty() { doc.append(parent, NodeKind::Text(text.to_string())); }
}

fn element_tag(doc: &Document, id: usize) -> Option<&str> {
    match &doc.nodes[id].kind { NodeKind::Element { tag, .. } => Some(tag), _ => None }
}

fn is_raw_text(tag: &str) -> bool { matches!(tag, "style" | "script" | "textarea" | "title") }
fn is_void(tag: &str) -> bool { matches!(tag, "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta" | "param" | "source" | "track" | "wbr") }

fn parse_start_tag(raw: &str) -> (String, BTreeMap<String, String>) {
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() { i += 1; }
    let tag = raw[..i].to_ascii_lowercase();
    let mut attrs = BTreeMap::new();
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b'/') { i += 1; }
        if i >= bytes.len() { break; }
        let start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'=' && bytes[i] != b'/' { i += 1; }
        if start == i { i += 1; continue; }
        let key = raw[start..i].to_ascii_lowercase();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() { i += 1; }
        let mut value = String::new();
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() { i += 1; }
            if i < bytes.len() && (bytes[i] == b'\'' || bytes[i] == b'"') {
                let quote = bytes[i];
                i += 1;
                let start_value = i;
                while i < bytes.len() && bytes[i] != quote { i += 1; }
                value = decode_entities(&raw[start_value..i]);
                if i < bytes.len() { i += 1; }
            } else {
                let start_value = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() { i += 1; }
                value = decode_entities(&raw[start_value..i]);
            }
        }
        attrs.entry(key).or_insert(value);
    }
    (tag, attrs)
}

fn decode_entities(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(index) = rest.find('&') {
        output.push_str(&rest[..index]);
        let entity_start = index + 1;
        let Some(end) = rest[entity_start..].find(';') else { output.push('&'); rest = &rest[entity_start..]; continue };
        let entity = &rest[entity_start..entity_start + end];
        let decoded = match entity {
            "amp" => Some('&'), "lt" => Some('<'), "gt" => Some('>'), "quot" => Some('"'),
            "apos" | "#39" => Some('\''), "nbsp" => Some('\u{00a0}'), "copy" => Some('\u{00a9}'), "reg" => Some('\u{00ae}'),
            _ if entity.starts_with("#x") || entity.starts_with("#X") => u32::from_str_radix(&entity[2..], 16).ok().and_then(char::from_u32),
            _ if entity.starts_with('#') => entity[1..].parse::<u32>().ok().and_then(char::from_u32),
            _ => None,
        };
        if let Some(ch) = decoded {
            output.push(ch);
            rest = &rest[entity_start + end + 1..];
        } else {
            output.push('&');
            rest = &rest[entity_start..];
        }
    }
    output.push_str(rest);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_text(doc: &Document, expected: &str) -> bool {
        doc.nodes.iter().any(|node| matches!(&node.kind, NodeKind::Text(text) if text == expected))
    }

    #[test]
    fn preserves_quoted_greater_than_in_attributes() {
        let doc = parse("<a title=\"1 > 0\" href='x'>link</a>");
        assert!(doc.nodes.iter().any(|node| matches!(&node.kind, NodeKind::Element { tag, attributes } if tag == "a" && attributes.get("title").is_some_and(|v| v == "1 > 0"))));
        assert!(has_text(&doc, "link"));
    }

    #[test]
    fn handles_raw_text_and_named_numeric_entities() {
        let doc = parse("<style>.x { content: '<tag>' }</style><script>if (a < b) run()</script><p>&nbsp;&#x1F680; &#169;</p>");
        assert!(has_text(&doc, ".x { content: '<tag>' }"));
        assert!(has_text(&doc, "if (a < b) run()"));
        assert!(has_text(&doc, "\u{00a0}🚀 ©"));
    }

    #[test]
    fn implicitly_closes_paragraph_and_list_item() {
        let doc = parse("<p>one<p>two<ul><li>a<li>b</ul>");
        let paragraphs: Vec<usize> = doc.nodes.iter().enumerate().filter_map(|(id, node)| match &node.kind { NodeKind::Element { tag, .. } if tag == "p" => Some(id), _ => None }).collect();
        let items: Vec<usize> = doc.nodes.iter().enumerate().filter_map(|(id, node)| match &node.kind { NodeKind::Element { tag, .. } if tag == "li" => Some(id), _ => None }).collect();
        assert_eq!(paragraphs.len(), 2);
        assert_eq!(items.len(), 2);
        assert_ne!(doc.nodes[paragraphs[0]].parent, Some(paragraphs[1]));
        assert_ne!(doc.nodes[items[0]].parent, Some(items[1]));
    }

    #[test]
    fn malformed_less_than_does_not_stall_and_unknown_entity_is_preserved() {
        let doc = parse("text < unfinished &unknown;");
        assert!(doc.nodes.iter().any(|node| matches!(&node.kind, NodeKind::Text(text) if text.contains("< unfinished") && text.contains("&unknown;"))));
    }
}
