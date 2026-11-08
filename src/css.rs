use std::collections::BTreeMap;

use crate::dom::{Document, NodeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display { Block, Inline, None }

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub display: Display,
    pub color: String,
    pub background: Option<String>,
    pub font_size: f32,
    pub margin: f32,
    pub padding: f32,
}

impl Default for ComputedStyle {
    fn default() -> Self { Self { display: Display::Inline, color: "#111111".into(), background: None, font_size: 16.0, margin: 0.0, padding: 0.0 } }
}

#[derive(Debug, Clone)]
struct Rule { selector: String, declarations: BTreeMap<String, String> }

#[derive(Debug, Clone, Default)]
pub struct StyleSheet { rules: Vec<Rule> }

impl StyleSheet {
    pub fn parse(source: &str) -> Self {
        let cleaned = strip_comments(source);
        let mut rules = Vec::new();
        for chunk in cleaned.split('}') {
            let Some((selectors, body)) = chunk.split_once('{') else { continue };
            let declarations = parse_declarations(body);
            for selector in selectors.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                rules.push(Rule { selector: selector.to_string(), declarations: declarations.clone() });
            }
        }
        Self { rules }
    }

    pub fn compute(&self, doc: &Document, id: usize, inherited: &ComputedStyle) -> ComputedStyle {
        let mut style = inherited.clone();
        style.background = None;
        style.margin = 0.0;
        style.padding = 0.0;
        if let NodeKind::Element { tag, .. } = &doc.nodes[id].kind {
            style.display = if is_block_tag(tag) { Display::Block } else { Display::Inline };
            for rule in &self.rules {
                if selector_matches(doc, id, &rule.selector) { apply(&mut style, &rule.declarations); }
            }
            if let Some(inline) = doc.element_attribute(id, "style") { apply(&mut style, &parse_declarations(inline)); }
        }
        style
    }
}

fn is_block_tag(tag: &str) -> bool { matches!(tag, "address" | "article" | "blockquote" | "body" | "div" | "footer" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "header" | "hr" | "li" | "main" | "ol" | "p" | "section" | "ul" | "pre") }

fn strip_comments(input: &str) -> String {
    let mut out = String::new(); let mut rest = input;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        if let Some(end) = after.find("*/") { rest = &after[end + 2..]; } else { return out; }
    }
    out.push_str(rest); out
}

fn parse_declarations(body: &str) -> BTreeMap<String, String> {
    body.split(';').filter_map(|part| {
        let (key, value) = part.split_once(':')?;
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim().to_string();
        if key.is_empty() || value.is_empty() { None } else { Some((key, value)) }
    }).collect()
}

fn apply(style: &mut ComputedStyle, declarations: &BTreeMap<String, String>) {
    for (key, value) in declarations {
        match key.as_str() {
            "display" => style.display = match value.as_str() { "none" => Display::None, "block" | "flex" | "grid" => Display::Block, "inline" | "inline-block" => Display::Inline, _ => style.display },
            "color" => style.color = value.clone(),
            "background" | "background-color" => style.background = Some(value.clone()),
            "font-size" => if let Some(size) = parse_length(value) { style.font_size = size.clamp(1.0, 256.0); },
            "margin" | "padding" => if let Some(length) = parse_length(value) { if key == "margin" { style.margin = length.max(0.0); } else { style.padding = length.max(0.0); } },
            _ => {}
        }
    }
}

fn parse_length(value: &str) -> Option<f32> {
    let value = value.trim();
    let numeric = value.strip_suffix("px").unwrap_or(value);
    numeric.parse::<f32>().ok().filter(|n| n.is_finite())
}

fn selector_matches(doc: &Document, id: usize, selector: &str) -> bool {
    let selector = selector.trim();
    if selector.is_empty() || selector.contains([' ', '>', '+', '~', ':', '[']) { return false; }
    let NodeKind::Element { tag, .. } = &doc.nodes[id].kind else { return false };
    let mut rest = selector;
    if let Some(hash) = rest.find('#') {
        let end = rest[hash + 1..].find(|c: char| c == '.' || c == '#').map_or(rest.len(), |n| hash + 1 + n);
        if doc.element_attribute(id, "id") != Some(&rest[hash + 1..end]) { return false; }
        rest = &rest[..hash].to_string();
    }
    let classes: Vec<&str> = rest.split('.').skip(1).collect();
    if let Some(class_attr) = doc.element_attribute(id, "class") {
        let present: Vec<&str> = class_attr.split_whitespace().collect();
        if classes.iter().any(|class| !present.contains(class)) { return false; }
    } else if !classes.is_empty() { return false; }
    let tag_part = rest.split('.').next().unwrap_or("");
    tag_part.is_empty() || tag_part == "*" || tag_part.eq_ignore_ascii_case(tag)
}
