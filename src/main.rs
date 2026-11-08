//! Experimental browser-engine building blocks for Supernova.
//! This crate is a learning/prototyping project, not a standards-compliant or secure browser.

pub mod browser;
pub mod css;
pub mod dom;
pub mod engine;
pub mod html;
pub mod layout;
pub mod paint;

pub use browser::BrowserSession;
pub use engine::{BrowserEngine, EngineError};

#[cfg(test)]
mod tests {
    use super::BrowserEngine;

    #[test]
    fn parses_and_renders_html() {
        let engine = BrowserEngine::new();
        let svg = engine.render_html("<h1 class='hero'>Hello</h1><p>World</p>");
        assert!(svg.contains("Hello"));
        assert!(svg.contains("World"));
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn escapes_text_in_svg() {
        let engine = BrowserEngine::new();
        let svg = engine.render_html("<p>A &amp; B</p>");
        assert!(svg.contains("A &amp; B"));
        assert!(!svg.contains("A & B"));
    }

    #[test]
    fn parses_closing_tags_and_attributes() {
        let doc = crate::html::parse("<div id='x'><br><p>ok</p></div>");
        assert!(doc.nodes.iter().any(|n| matches!(&n.kind, crate::dom::NodeKind::Element { tag, .. } if tag == "br")));
        assert!(doc.nodes.iter().any(|n| matches!(&n.kind, crate::dom::NodeKind::Element { attributes, .. } if attributes.get("id").is_some_and(|v| v == "x"))));
    }
}
