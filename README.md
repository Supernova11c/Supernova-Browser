# Supernova Browser Engine

Supernova is an experimental browser-engine foundation written in Rust. It parses a limited HTML subset, applies basic CSS, lays out block and inline text flow, and exports an SVG display list.

> **Status:** prototype, not a production or Chromium-compatible browser. The HTML/CSS implementations are incomplete; JavaScript, networking, accessibility, media, security sandboxing, and web-platform conformance are not implemented. Do not use it to browse untrusted content.

## Build and run

Requires stable Rust (1.70+).

```sh
cargo test
cargo run -- page.html --output page.svg
cargo run -- --html '<h1>Hello, Supernova!</h1>' --output page.svg
cargo run -- page.html --dump-dom
```

The CLI reads a local UTF-8 file or inline HTML. SVG output can be opened in an image viewer or browser. `--dump-dom` prints the parsed tree. The library exposes `BrowserEngine` for rendering and `BrowserSession` for page parsing, simple selector queries, DOM access, and viewport changes.

## Current milestone

- Lightweight DOM with parent/child links and attribute maps.
- Forgiving HTML tokenizer/tree builder for basic tags, comments, raw-text elements, and a small set of named/numeric entities.
- Limited CSS declarations and selectors (tag, class, ID, descendant, and child) with a basic specificity/order cascade.
- Simple block layout, inline text flow/wrapping, margins, padding, backgrounds, and SVG serialization.
- Unit tests for parsing, selectors, query API, text escaping, and layout behavior.

These are hand-built subsets, not standards-conformant implementations. CSS layout remains approximate; font measurement is estimated, inline backgrounds are not painted, and SVG is the only output backend.

## Architecture

- `html`: tokenizer and DOM tree construction.
- `dom`: nodes, attributes, and tree traversal.
- `css`: limited stylesheet parsing, selector matching, and computed styles.
- `layout`: basic block/inline flow into drawing primitives.
- `paint`: SVG serialization.
- `browser`: page/session API.
- `engine`: pipeline orchestration.

## Roadmap

A complete browser requires standards-tested HTML/CSS, a JavaScript engine, networking and caching, image/font/media decoding, accessibility and input, strong process/security boundaries, platform rendering, and extensive conformance/performance tests. Build these as separately testable subsystems with explicit security and compatibility goals.
