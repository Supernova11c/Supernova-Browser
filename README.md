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
    use super::{BrowserEngine, BrowserSession};

    #[test]
    fn parses_and_renders_html() {
        let svg = BrowserEngine::new().render_html("<h1 class='hero'>Hello</h1><p>World</p>");
        assert!(svg.contains("Hello"));
        assert!(svg.contains("World"));
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn escapes_text_in_svg() {
        let svg = BrowserEngine::new().render_html("<p>A &amp; B</p>");
        assert!(svg.contains("A &amp; B"));
        assert!(!svg.contains("A & B"));
    }

    #[test]
    fn session_query_finds_matching_nodes() {
        let page = BrowserSession::from_html("<main><p class='note'>one</p><p>two</p></main>");
        assert_eq!(page.query("p.note").len(), 1);
        assert_eq!(page.query("main p").len(), 2);
    }

    #[test]
    fn session_history_tracks_navigation() {
        let mut page = BrowserSession::from_html("<p>one</p>");
        page.load_html("<p>two</p>");
        page.load_html("<p>three</p>");
        assert!(page.back());
        assert!(page.document().nodes.iter().any(|node| matches!(&node.kind, crate::dom::NodeKind::Text(text) if text == "two")));
        assert!(page.forward());
        assert!(page.document().nodes.iter().any(|node| matches!(&node.kind, crate::dom::NodeKind::Text(text) if text == "three")));
    }

    #[test]
    fn document_helpers_find_nodes_and_ancestors() {
        let doc = crate::html::parse("<main><section id='hero'><p>ok</p></section></main>");
        let hero = doc.find_by_id("hero").unwrap();
        let p = doc.find_all_by_tag("p")[0];
        assert_eq!(doc.ancestors_of(p), vec![hero, 1]);
        assert_eq!(doc.tag_name(hero), Some("section"));
    }
}
