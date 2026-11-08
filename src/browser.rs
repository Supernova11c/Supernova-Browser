use crate::{
    css::{selector_matches, StyleSheet},
    dom::{Document, NodeKind},
    engine::BrowserEngine,
    html,
};

#[derive(Debug, Clone)]
pub struct BrowserSession {
    engine: BrowserEngine,
    document: Document,
    stylesheet: StyleSheet,
}

impl BrowserSession {
    pub fn new() -> Self {
        Self {
            engine: BrowserEngine::new(),
            document: Document::new(),
            stylesheet: StyleSheet::default(),
        }
    }

    pub fn from_html(source: &str) -> Self {
        let document = html::parse(source);
        let stylesheet = StyleSheet::parse(&collect_style_blocks(&document));
        Self {
            engine: BrowserEngine::new(),
            document,
            stylesheet,
        }
    }

    pub fn from_document(document: Document) -> Self {
        let stylesheet = StyleSheet::parse(&collect_style_blocks(&document));
        Self {
            engine: BrowserEngine::new(),
            document,
            stylesheet,
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn stylesheet(&self) -> &StyleSheet {
        &self.stylesheet
    }

    pub fn render(&self) -> String {
        self.engine.render_document(&self.document)
    }

    pub fn dump_dom(&self) -> String {
        self.document.to_indented_string()
    }

    pub fn query(&self, selector: &str) -> Vec<usize> {
        let mut ids = Vec::new();
        for (id, node) in self.document.nodes.iter().enumerate() {
            if matches!(&node.kind, NodeKind::Element { .. }) && selector_matches(&self.document, id, selector) {
                ids.push(id);
            }
        }
        ids
    }

    pub fn set_viewport_width(&mut self, width: f32) -> Result<(), crate::engine::EngineError> {
        self.engine = self.engine.clone().with_viewport_width(width)?;
        Ok(())
    }
}

impl Default for BrowserSession {
    fn default() -> Self {
        Self::new()
    }
}

fn collect_style_blocks(document: &Document) -> String {
    let mut css = String::new();
    for (id, node) in document.nodes.iter().enumerate() {
        if let NodeKind::Element { tag, .. } = &node.kind {
            if tag == "style" {
                css.push_str(&document.text_content(id));
                css.push('\n');
            }
        }
    }
    css
}
