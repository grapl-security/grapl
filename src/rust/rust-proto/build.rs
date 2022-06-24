use std::{
    ffi::OsStr,
    path::Path,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = tonic_build::configure();
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=../Cargo.lock");
    println!("cargo:rerun-if-changed=build.rs");

    change_on_dir(Path::new("../../proto/"))?;

    let mut paths = Vec::new();
    get_proto_files("../../proto/graplinc", &mut paths)?;

    config
        .build_client(true)
        .build_server(true)
        .compile(&paths[..], &["../../proto/".to_string()])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
    Ok(())
}

fn get_proto_files(path: &str, paths: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.metadata()?.is_file() {
            if let Some("proto") = entry.path().extension().and_then(OsStr::to_str) {
                paths.push(entry.path().display().to_string());
            }
        } else {
            let path = entry.path().display().to_string();
            get_proto_files(&path, paths)?;
        }
    }
    Ok(())
}

fn change_on_dir(root_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    for entry in std::fs::read_dir(current_dir.join(root_dir))? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            change_on_dir(&entry.path())?;
        }
        if metadata.is_file() {
            let path = entry.path();
            let path = path.canonicalize()?;
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    Ok(())
}
