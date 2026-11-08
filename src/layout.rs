use crate::{css::{ComputedStyle, Display, StyleSheet}, dom::{Document, NodeKind}};

#[derive(Debug, Clone, PartialEq)]
pub struct Rect { pub x: f32, pub y: f32, pub width: f32, pub height: f32, pub color: String }
#[derive(Debug, Clone, PartialEq)]
pub struct TextRun { pub x: f32, pub y: f32, pub text: String, pub size: f32, pub weight: u16, pub color: String }
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DisplayList { pub rects: Vec<Rect>, pub text: Vec<TextRun>, pub width: f32, pub height: f32 }

#[derive(Debug, Clone)]
struct Flow {
    start_x: f32,
    x: f32,
    y: f32,
    width: f32,
    line_height: f32,
    has_content: bool,
}

impl Flow {
    fn new(x: f32, y: f32, width: f32, font_size: f32) -> Self {
        Self { start_x: x, x, y, width: width.max(1.0), line_height: font_size * 1.35, has_content: false }
    }
    fn finish_line(&mut self) {
        if self.has_content { self.y += self.line_height; self.x = self.start_x; self.has_content = false; self.line_height = 0.0; }
    }
    fn bottom(&self) -> f32 { self.y + if self.has_content { self.line_height } else { 0.0 } }
}

pub fn layout(doc: &Document, sheet: &StyleSheet, width: f32) -> DisplayList {
    let width = if width.is_finite() { width.max(1.0) } else { 800.0 };
    let mut list = DisplayList { width, ..DisplayList::default() };
    let inherited = ComputedStyle::default();
    let mut flow = Flow::new(8.0, 8.0, (width - 16.0).max(1.0), inherited.font_size);
    layout_children(doc, sheet, 0, &mut flow, &inherited, &mut list);
    list.height = (flow.bottom() + 8.0).max(64.0);
    list
}

fn layout_children(doc: &Document, sheet: &StyleSheet, parent: usize, flow: &mut Flow, inherited: &ComputedStyle, list: &mut DisplayList) {
    for &child in &doc.nodes[parent].children {
        match &doc.nodes[child].kind {
            NodeKind::Text(text) => append_text(text, inherited, flow, list),
            NodeKind::Element { tag, .. } => {
                let style = sheet.compute(doc, child, inherited);
                if style.display == Display::None { continue; }
                if tag == "br" {
                    if flow.has_content { flow.finish_line(); } else { flow.y += flow.line_height.max(inherited.font_size * 1.35); }
                    flow.line_height = inherited.font_size * 1.35;
                    continue;
                }
                if style.display == Display::Block { flow.finish_line(); layout_block(doc, sheet, child, &style, flow, list); }
                else { layout_children(doc, sheet, child, flow, &style, list); }
            }
            NodeKind::Document => {}
        }
    }
}

fn layout_block(doc: &Document, sheet: &StyleSheet, id: usize, style: &ComputedStyle, flow: &mut Flow, list: &mut DisplayList) {
    let margin = style.margin.max(0.0);
    let padding = style.padding.max(0.0);
    let box_x = flow.start_x;
    let box_y = flow.y + margin;
    let content_x = box_x + padding;
    let content_y = box_y + padding;
    let content_width = (flow.width - padding * 2.0).max(1.0);
    let mut inner = Flow::new(content_x, content_y, content_width, style.font_size);
    layout_children(doc, sheet, id, &mut inner, style, list);
    let content_bottom = inner.bottom();
    let content_height = (content_bottom - content_y).max(style.font_size * 1.35);
    let height = content_height + padding * 2.0;

    if let Some(color) = &style.background { list.rects.push(Rect { x: box_x, y: box_y, width: flow.width, height, color: color.clone() }); }
    if matches!(&doc.nodes[id].kind, NodeKind::Element { tag, .. } if tag == "hr") {
        list.rects.push(Rect { x: box_x, y: box_y + (height - 1.0) / 2.0, width: flow.width, height: 1.0, color: "#888888".into() });
    }
    if matches!(&doc.nodes[id].kind, NodeKind::Element { tag, .. } if tag == "li") {
        list.text.push(TextRun { x: (box_x + padding - 16.0).max(0.0), y: box_y + padding + style.font_size, text: "•".into(), size: style.font_size, weight: style.font_weight, color: style.color.clone() });
    }
    flow.y = box_y + height + margin;
    flow.x = flow.start_x;
    flow.has_content = false;
    flow.line_height = style.font_size * 1.35;
}

fn append_text(text: &str, style: &ComputedStyle, flow: &mut Flow, list: &mut DisplayList) {
    for word in text.split_whitespace() {
        let word_width = estimated_width(word, style.font_size);
        let space_width = if flow.has_content { estimated_width(" ", style.font_size) } else { 0.0 };
        if flow.has_content && flow.x + space_width + word_width > flow.start_x + flow.width { flow.finish_line(); }
        if flow.has_content { flow.x += estimated_width(" ", style.font_size); }
        list.text.push(TextRun { x: flow.x, y: flow.y + style.font_size, text: word.to_string(), size: style.font_size, weight: style.font_weight, color: style.color.clone() });
        flow.x += word_width;
        flow.has_content = true;
        flow.line_height = flow.line_height.max(style.font_size * 1.35);
    }
}

fn estimated_width(text: &str, font_size: f32) -> f32 { text.chars().count() as f32 * font_size * 0.56 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_text_and_applies_inline_font_weight() {
        let doc = crate::html::parse("<p>alpha <strong>beta</strong> gamma delta epsilon</p>");
        let list = layout(&doc, &StyleSheet::default(), 120.0);
        assert!(list.text.iter().any(|run| run.text == "alpha"));
        assert!(list.text.iter().any(|run| run.text == "beta" && run.weight == 700));
        assert!(list.text.iter().any(|run| run.text == "delta"));
    }

    #[test]
    fn computes_block_background_and_padding_box() {
        let doc = crate::html::parse("<div style='background: #ffeeaa; padding: 10px'>hello</div>");
        let list = layout(&doc, &StyleSheet::default(), 300.0);
        assert_eq!(list.rects.len(), 1);
        assert_eq!(list.rects[0].x, 8.0);
        assert_eq!(list.rects[0].width, 284.0);
        assert!(list.rects[0].height >= 40.0);
    }

    #[test]
    fn list_items_paint_bullets_and_hidden_content_is_skipped() {
        let doc = crate::html::parse("<ul><li>visible item</li></ul><p style='display:none'>secret</p>");
        let list = layout(&doc, &StyleSheet::default(), 300.0);
        assert!(list.text.iter().any(|run| run.text == "•"));
        assert!(!list.text.iter().any(|run| run.text == "secret"));
    }
}
