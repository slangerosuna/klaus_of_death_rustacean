use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=shaders");
    println!("cargo:rerun-if-changed=resources");
    println!("cargo:rerun-if-changed=build.rs");

    let base_target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or(".".to_string());
    let target_dir = std::env::var("CARGO_BUILD_TARGET_DIR").unwrap_or("target".to_string());
    let target_dir = format!("{}/{}", base_target_dir, target_dir);
    let profile = std::env::var("PROFILE").unwrap_or("debug".to_string());
    let target_dir = format!("{}/{}", target_dir, profile);

    let resources_dir = format!("{}/kod_resources", target_dir);
    let resources_dir = Path::new(resources_dir.as_str());

    if resources_dir.exists() {
        std::fs::remove_dir_all(resources_dir).unwrap();
    }

    std::fs::create_dir(resources_dir).unwrap();

    let resources_src_dir = Path::new("resources");
    copy_files(resources_src_dir, resources_dir);
}

fn copy_files(from: &Path, to: &Path) {
    let read_dir = std::fs::read_dir(from).unwrap();
    for entry in read_dir {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().unwrap();
            let new_dir = to.join(dir_name);
            std::fs::create_dir(&new_dir).unwrap();
            copy_files(&path, &new_dir);
        } else {
            let file_name = path.file_name().unwrap();
            let new_file = to.join(file_name);
            std::fs::copy(&path, &new_file).unwrap();
        }
    }
}
