use crate::{css::{ComputedStyle, Display, StyleSheet}, dom::{Document, NodeKind}};

#[derive(Debug, Clone, PartialEq)]
pub struct Rect { pub x: f32, pub y: f32, pub width: f32, pub height: f32, pub color: String }
#[derive(Debug, Clone, PartialEq)]
pub struct TextRun { pub x: f32, pub y: f32, pub text: String, pub size: f32, pub color: String }
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DisplayList { pub rects: Vec<Rect>, pub text: Vec<TextRun>, pub width: f32, pub height: f32 }

pub fn layout(doc: &Document, sheet: &StyleSheet, width: f32) -> DisplayList {
    let mut list = DisplayList { width: width.max(1.0), ..DisplayList::default() };
    let inherited = ComputedStyle::default();
    let bottom = layout_children(doc, sheet, 0, 8.0, 8.0, (width - 16.0).max(1.0), &inherited, &mut list);
    list.height = (bottom + 8.0).max(64.0);
    list
}

fn layout_children(doc: &Document, sheet: &StyleSheet, parent: usize, x: f32, mut y: f32, width: f32, inherited: &ComputedStyle, list: &mut DisplayList) -> f32 {
    let children = doc.nodes[parent].children.clone();
    for child in children {
        match &doc.nodes[child].kind {
            NodeKind::Text(text) => {
                let style = inherited;
                let lines = wrap_text(text, width, style.font_size);
                for line in &lines {
                    if !line.is_empty() { list.text.push(TextRun { x, y: y + style.font_size, text: line.clone(), size: style.font_size, color: style.color.clone() }); }
                    y += style.font_size * 1.35;
                }
                if lines.is_empty() { y += style.font_size * 1.35; }
            }
            NodeKind::Element { .. } => {
                let style = sheet.compute(doc, child, inherited);
                if style.display == Display::None { continue; }
                let margin = style.margin;
                y += margin;
                let child_x = x + style.padding;
                let child_width = (width - style.padding * 2.0).max(1.0);
                if let Some(color) = &style.background {
                    list.rects.push(Rect { x, y, width, height: style.font_size * 1.5 + style.padding * 2.0, color: color.clone() });
                }
                let before = y;
                let bottom = layout_children(doc, sheet, child, child_x, y + style.padding, child_width, &style, list);
                let height = (bottom - before + style.padding).max(style.font_size * 1.35);
                if let Some(rect) = list.rects.iter_mut().rev().find(|r| (r.x - x).abs() < f32::EPSILON && (r.y - y).abs() < f32::EPSILON) { rect.height = height; }
                y = before + height + margin;
            }
            NodeKind::Document => {}
        }
    }
    y
}

fn wrap_text(text: &str, width: f32, size: f32) -> Vec<String> {
    let max_chars = ((width / (size * 0.56).max(1.0)).floor() as usize).max(1);
    let mut lines = Vec::new(); let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > max_chars {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() { line.push(' '); }
        line.push_str(word);
        while line.chars().count() > max_chars {
            let tail: String = line.chars().skip(max_chars).collect();
            line = line.chars().take(max_chars).collect();
            lines.push(std::mem::replace(&mut line, tail));
        }
    }
    if !line.is_empty() { lines.push(line); }
    lines
}
