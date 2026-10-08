//! Turns the built web interface (`assets/ui/`, made by `Ui/` and committed) into a table of files that
//! `include_bytes!` puts into the binary, so the server needs no files at run time and a plain
//! `cargo install` needs no node.

use std::path::{Path, PathBuf};

fn collect(dir: &Path, base: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, base, out)?;
        } else {
            out.push(path.strip_prefix(base).unwrap_or(&path).to_path_buf());
        }
    }
    Ok(())
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

fn main() {
    let root =
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default()).join("assets/ui");
    println!("cargo:rerun-if-changed=assets/ui");
    println!("cargo:rerun-if-changed=build.rs");
    let mut files = Vec::new();
    if let Err(error) = collect(&root, &root, &mut files) {
        panic!(
            "cannot read {}: {error}. Build the interface with Code/Development/Web/build_Ui.sh",
            root.display()
        );
    }
    files.sort();
    assert!(
        files.iter().any(|f| f == Path::new("index.html")),
        "{} holds no index.html; build the interface with Code/Development/Web/build_Ui.sh",
        root.display()
    );
    let rows: Vec<String> = files
        .iter()
        .map(|file| {
            format!(
                "    ({:?}, {:?}, include_bytes!({:?})),",
                file.to_string_lossy().replace('\\', "/"),
                content_type(file),
                root.join(file).to_string_lossy()
            )
        })
        .collect();
    let table = format!(
        "pub(crate) const FILES: &[(&str, &str, &[u8])] = &[\n{}\n];\n",
        rows.join("\n")
    );
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default()).join("ui_files.rs");
    if let Err(error) = std::fs::write(&out, table) {
        panic!("cannot write {}: {error}", out.display());
    }
}
