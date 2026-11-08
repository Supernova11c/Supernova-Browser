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
            if let Some(end) = rest.find('>') {
                let raw = rest[1..end].trim();
                cursor += end + 1;
                if raw.is_empty() || raw.starts_with('!') || raw.starts_with('?') { continue; }
                if let Some(closing) = raw.strip_prefix('/') {
                    let name = closing.split_whitespace().next().unwrap_or("").to_ascii_lowercase();
                    if let Some(pos) = stack.iter().rposition(|&id| element_tag(&doc, id) == Some(name.as_str())) {
                        stack.truncate(pos.max(1));
                    }
                    continue;
                }
                let self_closing = raw.ends_with('/');
                let raw = raw.strip_suffix('/').unwrap_or(raw).trim_end();
                let (tag, attributes) = parse_start_tag(raw);
                if tag.is_empty() { continue; }
                let parent = *stack.last().unwrap_or(&0);
                let id = doc.append(parent, NodeKind::Element { tag: tag.clone(), attributes });
                if !self_closing && !is_void(&tag) {
                    stack.push(id);
                    if is_raw_text(&tag) {
                        let content_start = cursor;
                        let content_end = find_closing_tag(source, content_start, &tag).unwrap_or(source.len());
                        if content_end > content_start {
                            let content = &source[content_start..content_end];
                            let text = if matches!(tag.as_str(), "textarea" | "title") { decode_entities(content) } else { content.to_string() };
                            doc.append(id, NodeKind::Text(text));
                        }
                        cursor = content_end;
                    }
                }
                continue;
            }
        }
        let next = rest.find('<').unwrap_or(rest.len());
        let text = decode_entities(&rest[..next]);
        if !text.is_empty() {
            doc.append(*stack.last().unwrap_or(&0), NodeKind::Text(text));
        }
        cursor += next;
    }
    doc
}

fn find_closing_tag(source: &str, from: usize, tag: &str) -> Option<usize> {
    let closing = format!("</{tag}");
    let lower = source[to_lowerbound(source, from)..].to_ascii_lowercase();
    let needle = lower.as_bytes();
    let start = closing.to_ascii_lowercase();
    let pattern = start.as_bytes();
    let mut i = from;
    while i + pattern.len() <= source.len() {
        let window = &source.as_bytes()[i..i + pattern.len()];
        if window.eq_ignore_ascii_case(pattern) {
            return source[i..].find('>').map(|pos| i + pos + 1);
        }
        i += 1;
    }
    None
}

fn to_lowerbound(source: &str, from: usize) -> usize {
    source[..from].len()
}

fn element_tag(doc: &Document, id: usize) -> Option<&str> {
    match &doc.nodes[id].kind {
        NodeKind::Element { tag, .. } => Some(tag),
        _ => None,
    }
}

fn is_raw_text(tag: &str) -> bool {
    matches!(tag, "style" | "script" | "textarea" | "title")
}

fn is_void(tag: &str) -> bool {
    matches!(tag, "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta" | "param" | "source" | "track" | "wbr")
}

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
        let Some(end) = rest[entity_start..].find(';') else { output.push('&'); rest = &rest[entity_start..]; continue; };
        let entity = &rest[entity_start..entity_start + end];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" | "#39" => Some('\''),
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

pub fn selector_matches(doc: &Document, id: usize, selector: &str) -> bool {
    let normalized = selector.replace('>', " > ");
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    if tokens.is_empty() || tokens.iter().any(|token| *token == "+" || *token == "~" || token.starts_with(':') || token.starts_with('[')) {
        return false;
    }

    let mut current = id;
    let mut idx = tokens.len();
    if !matches_compound(doc, current, tokens[idx - 1]) { return false; }
    idx -= 1;

    while idx > 0 {
        let relation = tokens[idx - 1];
        if relation == ">" {
            idx -= 1;
            if idx == 0 { return false; }
            let wanted = tokens[idx - 1];
            let Some(parent) = doc.nodes[current].parent else { return false; };
            if !matches_compound(doc, parent, wanted) { return false; }
            current = parent;
            idx -= 1;
            continue;
        }

        idx -= 1;
        if idx == 0 { return false; }
        let wanted = tokens[idx - 1];
        let Some(mut ancestor) = doc.nodes[current].parent else { return false; };
        loop {
            if matches_compound(doc, ancestor, wanted) {
                current = ancestor;
                break;
            }
            let Some(parent) = doc.nodes[ancestor].parent else { return false; };
            ancestor = parent;
        }
        idx -= 1;
    }
    true
}

fn matches_compound(doc: &Document, id: usize, compound: &str) -> bool {
    let NodeKind::Element { tag, .. } = &doc.nodes[id].kind else { return false; };
    let bytes = compound.as_bytes();
    let mut tag_end = 0;
    while tag_end < bytes.len() && bytes[tag_end] != b'#' && bytes[tag_end] != b'.' {
        tag_end += 1;
    }
    if tag_end > 0 {
        let wanted = &compound[..tag_end];
        if wanted != "*" && !wanted.eq_ignore_ascii_case(tag) { return false; }
    }
    let mut i = tag_end;
    while i < bytes.len() {
        let marker = bytes[i];
        i += 1;
        let start = i;
        while i < bytes.len() && bytes[i] != b'#' && bytes[i] != b'.' {
            i += 1;
        }
        if start == i { return false; }
        let value = &compound[start..i];
        if marker == b'#' {
            if doc.element_attribute(id, "id") != Some(value) { return false; }
        } else {
            let Some(class_attr) = doc.element_attribute(id, "class") else { return false; };
            if !class_attr.split_whitespace().any(|class| class == value) { return false; }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_and_script_contents_are_raw_text() {
        let doc = parse("<style>.x { color: red }</style><script>if (a < b) x()</script><p>A &#x1F680;</p>");
        assert!(doc.nodes.iter().any(|node| matches!(&node.kind, NodeKind::Text(text) if text == ".x { color: red }")));
        assert!(doc.nodes.iter().any(|node| matches!(&node.kind, NodeKind::Text(text) if text == "if (a < b) x()")));
        assert!(doc.nodes.iter().any(|node| matches!(&node.kind, NodeKind::Text(text) if text == "A 🚀")));
    }
}
