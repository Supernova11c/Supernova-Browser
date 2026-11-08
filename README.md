# Supernova Browser Engine

Supernova is an experimental browser-engine foundation written in Rust. This initial project is intentionally small, dependency-free, and easy to build on: it parses a useful HTML subset, applies basic CSS, performs a simple vertical-flow layout, and exports the result as SVG.

> **Status:** prototype, not a production browser and not comparable to Chromium yet. It does not implement the web platform, JavaScript, networking, standards-compliant HTML/CSS, accessibility, sandboxing, or a hardened rendering pipeline. Do not use it to browse untrusted content.

## Build and run

Requires stable Rust (1.70 or newer).

```sh
cargo test
cargo run -- page.html --output page.svg
cargo run -- --html '<h1>Hello, Supernova!</h1>' --output page.svg
cargo run -- page.html --dump-dom
```

The input is a local UTF-8 file. SVG output can be opened in a regular image viewer or browser. `--dump-dom` prints the parsed tree and exits. With no arguments, the CLI prints usage.

## Current capabilities

- HTML elements, attributes, text, comments, void elements, and a forgiving stack-based tree builder.
- Basic tag, `.class`, and `#id` CSS selectors; simple declarations, inline styles, and embedded `<style>` blocks.
- A deliberately basic block-flow layout and SVG display-list output.
- Small unit tests and no third-party runtime dependencies.

## Architecture

- `html`: tokenizer and DOM tree construction.
- `css`: stylesheet parsing, selector matching, and computed styles.
- `layout`: basic flow layout into drawing primitives.
- `paint`: SVG serialization and escaping.
- `engine`: public API that connects the pipeline.

## Roadmap toward a real browser engine

A serious browser needs much more than a renderer: standards-based HTML parsing and CSS layout, a JavaScript engine, networking and caching, image/font/media decoding, accessibility and input, robust security boundaries, extensive web-platform tests, and performance profiling. These should be implemented as separately testable subsystems with explicit conformance and security goals. Supernova currently provides only a starting point for that work.
