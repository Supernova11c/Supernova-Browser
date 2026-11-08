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
    history: Vec<Document>,
    history_index: usize,
}

impl BrowserSession {
    pub fn new() -> Self {
        let document = Document::new();
        let stylesheet = StyleSheet::default();
        Self {
            engine: BrowserEngine::new(),
            document: document.clone(),
            stylesheet: stylesheet.clone(),
            history: vec![document],
            history_index: 0,
        }
    }

    pub fn from_html(source: &str) -> Self {
        let mut session = Self::new();
        session.load_html(source);
        session
    }

    pub fn from_document(document: Document) -> Self {
        let mut session = Self::new();
        session.load_document(document);
        session
    }

    pub fn load_html(&mut self, source: &str) {
        self.load_document(html::parse(source));
    }

    pub fn load_document(&mut self, document: Document) {
        let stylesheet = StyleSheet::parse(&collect_style_blocks(&document));
        if self.history.last().is_some_and(|current| current == &document) && self.history_index == self.history.len() - 1 {
            self.document = document;
            self.stylesheet = stylesheet;
            return;
        }
        self.history.truncate(self.history_index + 1);
        self.history.push(document.clone());
        self.history_index = self.history.len() - 1;
        self.document = document;
        self.stylesheet = stylesheet;
    }

    pub fn back(&mut self) -> bool {
        if self.history_index == 0 { return false; }
        self.history_index -= 1;
        self.sync_from_history();
        true
    }

    pub fn forward(&mut self) -> bool {
        if self.history_index + 1 >= self.history.len() { return false; }
        self.history_index += 1;
        self.sync_from_history();
        true
    }

    pub fn can_go_back(&self) -> bool { self.history_index > 0 }
    pub fn can_go_forward(&self) -> bool { self.history_index + 1 < self.history.len() }
    pub fn history_len(&self) -> usize { self.history.len() }
    pub fn current_index(&self) -> usize { self.history_index }

    pub fn document(&self) -> &Document { &self.document }
    pub fn stylesheet(&self) -> &StyleSheet { &self.stylesheet }
    pub fn render(&self) -> String { self.engine.render_document(&self.document) }
    pub fn dump_dom(&self) -> String { self.document.to_indented_string() }

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

    fn sync_from_history(&mut self) {
        let document = self.history[self.history_index].clone();
        let stylesheet = StyleSheet::parse(&collect_style_blocks(&document));
        self.document = document;
        self.stylesheet = stylesheet;
    }
}

impl Default for BrowserSession {
    fn default() -> Self { Self::new() }
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
