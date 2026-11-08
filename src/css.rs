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
struct Rule { selector: String, declarations: BTreeMap<String, String>, specificity: (usize, usize, usize), order: usize }

#[derive(Debug, Clone, Default)]
pub struct StyleSheet { rules: Vec<Rule> }

impl StyleSheet {
    /// Parse a deliberately limited CSS subset. Unsupported at-rules and selectors are ignored.
    pub fn parse(source: &str) -> Self {
        let cleaned = strip_comments(source);
        let mut rules = Vec::new();
        for chunk in cleaned.split('}') {
            let Some((selectors, body)) = chunk.split_once('{') else { continue };
            let declarations = parse_declarations(body);
            for selector in selectors.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                let order = rules.len();
                rules.push(Rule { selector: selector.to_string(), declarations: declarations.clone(), specificity: specificity(selector), order });
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
            style.display = if is_hidden_tag(tag) { Display::None } else if is_block_tag(tag) { Display::Block } else { Display::Inline };
            let mut matching: Vec<&Rule> = self.rules.iter().filter(|rule| selector_matches(doc, id, &rule.selector)).collect();
            matching.sort_by_key(|rule| (rule.specificity, rule.order));
            for rule in matching { apply(&mut style, &rule.declarations); }
            if let Some(inline) = doc.element_attribute(id, "style") { apply(&mut style, &parse_declarations(inline)); }
        }
        style
    }
}

fn is_hidden_tag(tag: &str) -> bool { matches!(tag, "head" | "style" | "script" | "meta" | "link" | "title" | "noscript") }
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
    let mut declarations = BTreeMap::new();
    for part in body.split(';') {
        let Some((key, value)) = part.split_once(':') else { continue };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim().trim_end_matches("!important").trim().to_string();
        if !key.is_empty() && !value.is_empty() { declarations.insert(key, value); }
    }
    declarations
}

fn apply(style: &mut ComputedStyle, declarations: &BTreeMap<String, String>) {
    for (key, value) in declarations {
        match key.as_str() {
            "display" => style.display = match value.as_str() { "none" => Display::None, "block" | "flex" | "grid" | "list-item" => Display::Block, "inline" | "inline-block" => Display::Inline, _ => style.display },
            "color" => style.color = value.clone(),
            "background" | "background-color" => style.background = Some(value.clone()),
            "font-size" => if let Some(size) = parse_length(value) { style.font_size = size.clamp(1.0, 256.0); },
            "margin" | "padding" => if let Some(length) = parse_length(value) { if key == "margin" { style.margin = length; } else { style.padding = length.max(0.0); } },
            _ => {}
        }
    }
}

fn parse_length(value: &str) -> Option<f32> {
    let value = value.trim();
    let numeric = value.strip_suffix("px").or_else(|| value.strip_suffix("pt")).unwrap_or(value);
    numeric.parse::<f32>().ok().filter(|n| n.is_finite())
}

fn specificity(selector: &str) -> (usize, usize, usize) {
    let mut ids = 0; let mut classes = 0; let mut tags = 0;
    let tokens = selector.split(|c: char| c.is_whitespace() || c == '>');
    for token in tokens {
        let mut expect_tag = true;
        for ch in token.chars() {
            match ch {
                '#' => { ids += 1; expect_tag = false; }
                '.' | ':' | '[' => { classes += 1; expect_tag = false; }
                '*' => expect_tag = false,
                _ if expect_tag => { tags += 1; expect_tag = false; }
                _ => {}
            }
        }
    }
    (ids, classes, tags)
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
    fn applies_specificity_and_inline_style() {
        let doc = crate::html::parse("<div id='target' class='card'>text</div>");
        let sheet = StyleSheet::parse("div { color: red } .card { color: blue } #target { color: green }");
        let mut style = sheet.compute(&doc, 1, &ComputedStyle::default());
        assert_eq!(style.color, "green");
        style = StyleSheet::parse("#target { color: green }").compute(&doc, 1, &style);
        assert_eq!(style.color, "green");
    }

    #[test]
    fn matches_descendant_and_child_selectors() {
        let doc = crate::html::parse("<main><section><p class='note'>ok</p></section></main>");
        let sheet = StyleSheet::parse("main p { color: red } main > p { background-color: blue }");
        let style = sheet.compute(&doc, 3, &ComputedStyle::default());
        assert_eq!(style.color, "red");
        assert_eq!(style.background, None);
    }
}
