use std::{env, fs, process};
use supernova_browser::{html, BrowserEngine};

fn main() {
    if let Err(message) = run() { eprintln!("supernova: {message}"); process::exit(2); }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut input = None;
    let mut inline_html = None;
    let mut output = None;
    let mut dump_dom = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--html" => inline_html = Some(args.next().ok_or("--html requires a string")?),
            "--output" | "-o" => output = Some(args.next().ok_or("--output requires a path")?),
            "--dump-dom" => dump_dom = true,
            "--help" | "-h" => { print_help(); return Ok(()); }
            _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}")),
            _ if input.is_none() => input = Some(arg),
            _ => return Err(format!("unexpected argument: {arg}")),
        }
    }
    let source = if let Some(html) = inline_html { html } else if let Some(path) = input {
        fs::read_to_string(&path).map_err(|e| format!("could not read {path}: {e}"))?
    } else { print_help(); return Ok(()); };
    let engine = BrowserEngine::new();
    if dump_dom {
        print!("{}", html::parse(&source).to_indented_string());
        return Ok(());
    }
    let svg = engine.render_html(&source);
    if let Some(path) = output { fs::write(&path, svg).map_err(|e| format!("could not write {path}: {e}"))?; }
    else { print!("{svg}"); }
    Ok(())
}

fn print_help() {
    println!("Supernova Browser Engine (experimental)\n\nUSAGE:\n  supernova <FILE> [--output <SVG>] [--dump-dom]\n  supernova --html <HTML> [--output <SVG>]\n");
}
