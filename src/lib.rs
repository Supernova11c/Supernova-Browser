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

    pub fn tag_name(&self, id: usize) -> Option<&str> {
        match &self.nodes.get(id)?.kind {
            NodeKind::Element { tag, .. } => Some(tag),
            _ => None,
        }
    }

    pub fn element_attribute(&self, id: usize, name: &str) -> Option<&str> {
        match &self.nodes.get(id)?.kind {
            NodeKind::Element { attributes, .. } => attributes.get(name).map(String::as_str),
            _ => None,
        }
    }

    pub fn children_of(&self, id: usize) -> &[usize] {
        self.nodes.get(id).map(|node| node.children.as_slice()).unwrap_or(&[])
    }

    pub fn ancestors_of(&self, id: usize) -> Vec<usize> {
        let mut ancestors = Vec::new();
        let mut current = self.nodes.get(id).and_then(|node| node.parent);
        while let Some(parent) = current {
            ancestors.push(parent);
            current = self.nodes.get(parent).and_then(|node| node.parent);
        }
        ancestors
    }

    pub fn descendants_of(&self, id: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut stack = self.children_of(id).to_vec();
        while let Some(current) = stack.pop() {
            out.push(current);
            stack.extend_from_slice(self.children_of(current));
        }
        out
    }

    pub fn find_by_id(&self, id: &str) -> Option<usize> {
        self.nodes.iter().enumerate().find_map(|(index, node)| match &node.kind {
            NodeKind::Element { attributes, .. } if attributes.get("id").is_some_and(|value| value == id) => Some(index),
            _ => None,
        })
    }

    pub fn find_all_by_tag(&self, tag: &str) -> Vec<usize> {
        self.nodes.iter().enumerate().filter_map(|(index, node)| match &node.kind {
            NodeKind::Element { tag: current, .. } if current.eq_ignore_ascii_case(tag) => Some(index),
            _ => None,
        }).collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_nodes_by_id_and_tag() {
        let mut doc = Document::new();
        let root = doc.append(0, NodeKind::Element { tag: "main".into(), attributes: BTreeMap::new() });
        let first = doc.append(root, NodeKind::Element { tag: "p".into(), attributes: BTreeMap::from([("id".into(), "lead".into())]) });
        let second = doc.append(root, NodeKind::Element { tag: "p".into(), attributes: BTreeMap::new() });
        let nested = doc.append(second, NodeKind::Element { tag: "span".into(), attributes: BTreeMap::new() });

        assert_eq!(doc.find_by_id("lead"), Some(first));
        assert_eq!(doc.find_all_by_tag("p"), vec![first, second]);
        assert_eq!(doc.ancestors_of(nested), vec![second, root]);
        assert!(doc.descendants_of(root).contains(&nested));
    }
}
