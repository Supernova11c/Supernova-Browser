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
    pub font_weight: u16,
    pub margin: f32,
    pub padding: f32,
    pub border_width: f32,
    pub box_width: f32,
    pub box_height: f32,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Inline,
            color: "#111111".into(),
            background: None,
            font_size: 16.0,
            font_weight: 400,
            margin: 0.0,
            padding: 0.0,
            border_width: 0.0,
            box_width: 0.0,
            box_height: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
struct Rule { selector: String, declarations: BTreeMap<String, String>, specificity: (usize, usize, usize), order: usize }

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
        style.border_width = 0.0;
        style.box_width = 0.0;
        style.box_height = 0.0;
        if let NodeKind::Element { tag, .. } = &doc.nodes[id].kind {
            style.display = if is_hidden_tag(tag) { Display::None } else if is_block_tag(tag) { Display::Block } else { Display::Inline };
            set_user_agent_defaults(tag, &mut style);
            let mut matching: Vec<&Rule> = self.rules.iter().filter(|rule| selector_matches(doc, id, &rule.selector)).collect();
            matching.sort_by_key(|rule| (rule.specificity, rule.order));
            for rule in matching { apply(&mut style, &rule.declarations); }
            if let Some(inline) = doc.element_attribute(id, "style") { apply(&mut style, &parse_declarations(inline)); }
        }
        style
    }
}

fn set_user_agent_defaults(tag: &str, style: &mut ComputedStyle) {
    match tag {
        "h1" => { style.font_size = 32.0; style.font_weight = 700; style.margin = 16.0; }
        "h2" => { style.font_size = 24.0; style.font_weight = 700; style.margin = 14.0; }
        "h3" => { style.font_size = 20.0; style.font_weight = 700; style.margin = 12.0; }
        "h4" | "h5" | "h6" => { style.font_weight = 700; style.margin = 10.0; }
        "p" | "blockquote" => style.margin = 8.0,
        "ul" | "ol" => style.margin = 8.0,
        "strong" | "b" => style.font_weight = 700,
        _ => {}
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
            "font-weight" => style.font_weight = match value.as_str() { "normal" => 400, "bold" | "bolder" => 700, "lighter" => 300, _ => value.parse::<u16>().ok().filter(|weight| (100..=900).contains(weight)).unwrap_or(style.font_weight) },
            "margin" | "padding" | "border-width" => if let Some(length) = parse_length(value) { match key.as_str() { "margin" => style.margin = length, "padding" => style.padding = length.max(0.0), "border-width" => style.border_width = length.max(0.0), _ => {} } },
            "width" => style.box_width = parse_length(value).unwrap_or(style.box_width),
            "height" => style.box_height = parse_length(value).unwrap_or(style.box_height),
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
    for token in selector.split(|c: char| c.is_whitespace() || c == '>' || c == '+' || c == '~') {
        if token.is_empty() { continue; }
        let mut saw_piece = false;
        let mut tag = false;
        for ch in token.chars() {
            match ch {
                '#' => { ids += 1; saw_piece = true; tag = false; }
                '.' => { classes += 1; saw_piece = true; tag = false; }
                ':' | '[' => { classes += 1; saw_piece = true; }
                _ if !saw_piece && !ch.is_ascii_whitespace() => { tag = true; }
                _ => {}
            }
        }
        if tag { tags += 1; }
    }
    (ids, classes, tags)
}

pub fn selector_matches(doc: &Document, id: usize, selector: &str) -> bool {
    let selector = selector.trim();
    if selector.is_empty() { return false; }
    let mut parts: Vec<&str> = selector.split_whitespace().collect();
    if parts.is_empty() { return false; }
    if parts.len() == 1 {
        return match_selector_component(doc, id, parts[0]);
    }
    let mut current = id;
    let mut index = 0usize;
    while index < parts.len() {
        if parts[index] == ">" { index += 1; continue; }
        if !match_selector_component(doc, current, parts[index]) { return false; }
        index += 1;
        if index >= parts.len() { break; }
        let relation = parts[index];
        if relation == ">" {
            if let Some(parent) = doc.nodes[current].parent {
                current = parent;
                index += 1;
            } else { return false; }
        } else {
            let mut ancestor = match doc.nodes[current].parent { Some(p) => p, None => return false };
            let mut matched = false;
            while let Some(parent) = doc.nodes.get(ancestor) {
                if match_selector_component(doc, ancestor, relation) {
                    current = ancestor;
                    matched = true;
                    break;
                }
                match parent.parent { Some(next) => ancestor = next, None => break };
            }
            if !matched { return false; }
            index += 1;
        }
    }
    true
}

fn match_selector_component(doc: &Document, id: usize, selector: &str) -> bool {
    let NodeKind::Element { tag, attributes } = &doc.nodes[id].kind else { return false; };
    let selector = selector.trim();
    if selector.is_empty() { return false; }
    let mut tag_name = "";
    let mut simple = String::new();
    let mut idx = 0usize;
    while idx < selector.len() {
        let ch = selector.as_bytes()[idx] as char;
        if ch == '#' || ch == '.' {
            break;
        }
        tag_name.push(ch);
        idx += 1;
    }
    if !tag_name.is_empty() && !tag_name.eq_ignore_ascii_case(tag) && tag_name != "*" { return false; }
    simple = selector[idx..].to_string();
    let mut id_matches = true;
    let mut class_matches = true;
    if let Some(hash_pos) = simple.find('#') {
        let value = &simple[hash_pos + 1..];
        id_matches = attributes.get("id").is_some_and(|v| v == value);
    }
    if let Some(dot_start) = simple.rfind('.') {
        let classes = &simple[dot_start + 1..];
        class_matches = classes.split('.').all(|class| attributes.get("class").is_some_and(|value| value.split_whitespace().any(|item| item == class)));
    }
    if selector.contains('#') || selector.contains('.') {
        return id_matches && class_matches;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_agent_heading_defaults_and_css_overrides_work() {
        let doc = crate::html::parse("<h1 id='title'>Hello</h1>");
        let style = StyleSheet::parse("#title { font-size: 40px; font-weight: 500 }").compute(&doc, 1, &ComputedStyle::default());
        assert_eq!(style.font_size, 40.0);
        assert_eq!(style.font_weight, 500);
        let defaults = StyleSheet::default().compute(&doc, 1, &ComputedStyle::default());
        assert_eq!(defaults.font_size, 32.0);
        assert_eq!(defaults.font_weight, 700);
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
