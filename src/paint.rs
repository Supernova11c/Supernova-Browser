use crate::layout::DisplayList;

pub fn to_svg(list: &DisplayList) -> String {
    let mut svg = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n<rect width=\"100%\" height=\"100%\" fill=\"white\"/>\n", number(list.width), number(list.height), number(list.width), number(list.height));
    for rect in &list.rects {
        svg.push_str(&format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"/>\n", number(rect.x), number(rect.y), number(rect.width), number(rect.height), escape(&rect.color)));
    }
    for run in &list.text {
        svg.push_str(&format!("<text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"{}\" font-weight=\"{}\" fill=\"{}\">{}</text>\n", number(run.x), number(run.y), number(run.size), run.weight, escape(&run.color), escape(&run.text)));
    }
    svg.push_str("</svg>\n");
    svg
}

fn number(value: f32) -> String { format!("{}", value.max(0.0)) }
fn escape(input: &str) -> String { input.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;") }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{DisplayList, TextRun};

    #[test]
    fn serializes_font_weight_and_escapes_text() {
        let list = DisplayList { width: 100.0, height: 50.0, text: vec![TextRun { x: 1.0, y: 20.0, text: "A < B".into(), size: 16.0, weight: 700, color: "#000".into() }], rects: vec![] };
        let svg = to_svg(&list);
        assert!(svg.contains("font-weight=\"700\""));
        assert!(svg.contains("A &lt; B"));
    }
}
