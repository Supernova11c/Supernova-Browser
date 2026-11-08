use std::fmt;

use crate::{css::{ComputedStyle, StyleSheet}, dom::{Document, NodeKind}, html, layout, paint};

#[derive(Debug)]
pub enum EngineError { InvalidViewport, Render(String) }
impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self { Self::InvalidViewport => write!(f, "viewport width must be finite and positive"), Self::Render(message) => f.write_str(message) }
    }
}
impl std::error::Error for EngineError {}

#[derive(Debug, Clone)]
pub struct BrowserEngine { viewport_width: f32 }

impl BrowserEngine {
    pub fn new() -> Self { Self { viewport_width: 800.0 } }
    pub fn with_viewport_width(mut self, width: f32) -> Result<Self, EngineError> {
        if !width.is_finite() || width <= 0.0 { return Err(EngineError::InvalidViewport); }
        self.viewport_width = width; Ok(self)
    }
    pub fn parse(&self, source: &str) -> Document { html::parse(source) }
    pub fn render_html(&self, source: &str) -> String {
        let doc = self.parse(source);
        let sheet = stylesheet_for(&doc);
        paint::to_svg(&layout::layout(&doc, &sheet, self.viewport_width))
    }
    pub fn render_document(&self, doc: &Document) -> String {
        let sheet = stylesheet_for(doc);
        paint::to_svg(&layout::layout(doc, &sheet, self.viewport_width))
    }
}
impl Default for BrowserEngine { fn default() -> Self { Self::new() } }

fn stylesheet_for(doc: &Document) -> StyleSheet {
    let mut source = String::new();
    for (id, node) in doc.nodes.iter().enumerate() {
        if matches!(&node.kind, NodeKind::Element { tag, .. } if tag == "style") { source.push_str(&doc.text_content(id)); source.push('\n'); }
    }
    StyleSheet::parse(&source)
}

#[allow(dead_code)]
fn _default_style() -> ComputedStyle { ComputedStyle::default() }
