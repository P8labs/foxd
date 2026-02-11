use std::path::Path;

fn main() {
    let console_dir = Path::new("../console/build");
    if !console_dir.exists() {
        std::fs::create_dir_all(console_dir).expect("Failed to create console build directory");
    }
}
