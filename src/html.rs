use std::collections::BTreeMap;

use crate::dom::{Document, NodeKind};

/// Parse a forgiving subset of HTML into a lightweight DOM. This is not the HTML Living Standard parser.
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
                if !self_closing && !is_void(&tag) { stack.push(id); }
                continue;
            }
        }
        let next = rest.find('<').unwrap_or(rest.len());
        let text = decode_entities(&rest[..next]);
        if !text.is_empty() { doc.append(*stack.last().unwrap_or(&0), NodeKind::Text(text)); }
        cursor += next;
    }
    doc
}

fn element_tag(doc: &Document, id: usize) -> Option<&str> {
    match &doc.nodes[id].kind { NodeKind::Element { tag, .. } => Some(tag), _ => None }
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
                let quote = bytes[i]; i += 1; let start_value = i;
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
    input.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'").replace("&apos;", "'")
}
