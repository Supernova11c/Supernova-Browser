# Supernova Browser Engine

Supernova is an experimental browser-engine foundation written in Rust. It parses a limited HTML subset, applies basic CSS, lays out block and inline text flow, and exports an SVG display list.

> **Status:** prototype, not a production or Chromium-compatible browser. HTML/CSS implementations are incomplete; JavaScript, networking, accessibility, media, security sandboxing, and web-platform conformance are not implemented. Do not use it to browse untrusted content.

## Build and run

Requires stable Rust (1.70+).

```sh
cargo test
cargo run -- page.html --output page.svg
cargo run -- --html '<h1>Hello, Supernova!</h1>' --output page.svg
cargo run -- page.html --dump-dom
```

## Current milestones

- Lightweight DOM and forgiving HTML subset parser, including quoted `>` attributes, raw-text elements, selected named/numeric entities, and basic implied paragraph/list-item closing.
- CSS subset with tag/class/ID, descendant and child selectors, cascade ordering, colors, display, font size/weight, margins and padding.
- Approximate block and inline text flow, wrapping, hidden elements, backgrounds, `<br>`, and list-item bullets.
- Browser-session navigation and document history with back/forward traversal.
- SVG display-list output and unit tests for parsing, styling, layout, and escaping.

These are hand-built subsets, not standards-conformant implementations. Font measurement is estimated, list styles are basic, and SVG is the only output backend.

## Architecture and roadmap

Subsystems live in `src/html.rs`, `src/dom.rs`, `src/css.rs`, `src/layout.rs`, `src/paint.rs`, and `src/engine.rs`. A complete browser also requires standards-tested HTML/CSS, JavaScript, networking and caching, image/font/media decoding, accessibility and input, strong process/security boundaries, platform rendering, and extensive conformance/performance tests.
