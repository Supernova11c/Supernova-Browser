use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Document,
    Element { tag: String, attributes: BTreeMap<String, String> },
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub kind: NodeKind,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub nodes: Vec<Node>,
}

impl Document {
    pub fn new() -> Self {
        Self { nodes: vec![Node { kind: NodeKind::Document, parent: None, children: Vec::new() }] }
    }

    pub fn append(&mut self, parent: usize, kind: NodeKind) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node { kind, parent: Some(parent), children: Vec::new() });
        self.nodes[parent].children.push(id);
        id
    }

    pub fn element_attribute(&self, id: usize, name: &str) -> Option<&str> {
        match &self.nodes.get(id)?.kind {
            NodeKind::Element { attributes, .. } => attributes.get(name).map(String::as_str),
            _ => None,
        }
    }

    pub fn text_content(&self, id: usize) -> String {
        fn collect(doc: &Document, id: usize, out: &mut String) {
            let Some(node) = doc.nodes.get(id) else { return };
            if let NodeKind::Text(text) = &node.kind { out.push_str(text); }
            for &child in &node.children { collect(doc, child, out); }
        }
        let mut out = String::new();
        collect(self, id, &mut out);
        out
    }

    pub fn to_indented_string(&self) -> String {
        fn write_node(doc: &Document, id: usize, depth: usize, out: &mut String) {
            let node = &doc.nodes[id];
            out.push_str(&"  ".repeat(depth));
            match &node.kind {
                NodeKind::Document => out.push_str("#document\n"),
                NodeKind::Text(text) => { out.push_str(&format!("#text {:?}\n", text)); }
                NodeKind::Element { tag, attributes } => {
                    out.push('<'); out.push_str(tag);
                    for (key, value) in attributes { out.push_str(&format!(" {}=\"{}\"", key, value)); }
                    out.push_str(">\n");
                }
            }
            for &child in &node.children { write_node(doc, child, depth + 1, out); }
        }
        let mut out = String::new();
        write_node(self, 0, 0, &mut out);
        out
    }
}

impl Default for Document {
    fn default() -> Self { Self::new() }
}
