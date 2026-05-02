use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let os_root = manifest_dir.join("web/os");
    let index = os_root.join("index.html");
    if !index.exists() {
        panic!(
            "missing vendored TytusOS bundle at {} — run services/tytus-os/app npm build and copy dist into services/tytus-cli/tray/web/os",
            os_root.display()
        );
    }

    println!("cargo:rerun-if-changed={}", os_root.display());

    let mut files = Vec::new();
    collect_files(&os_root, &os_root, &mut files);
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let out_path = out_dir.join("os_assets.rs");
    let mut out = fs::File::create(&out_path).expect("create os_assets.rs");
    writeln!(out, "pub static OS_ASSETS: &[(&str, &[u8])] = &[").unwrap();
    for (rel, abs) in files {
        writeln!(
            out,
            "    ({:?}, include_bytes!({:?})),",
            format!("/{rel}"),
            abs.to_string_lossy()
        )
        .unwrap();
    }
    writeln!(out, "];").unwrap();
}

fn collect_files(root: &Path, dir: &Path, files: &mut Vec<(String, PathBuf)>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| {
        panic!("failed to read {}: {e}", dir.display());
    });
    for entry in entries {
        let entry = entry.expect("read_dir entry");
        let path = entry.path();
        if path.is_symlink() {
            continue;
        }
        if path.is_dir() {
            collect_files(root, &path, files);
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .expect("asset under root")
            .to_string_lossy()
            .replace('\\', "/");
        files.push((rel, path));
    }
}
