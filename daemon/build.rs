use std::path::Path;
use std::process::Command;

fn main() {
    let console_dir = Path::new("../console/build");
    let console_index = console_dir.join("index.html");

    if !console_index.exists() {
        println!("cargo:warning=Console not found, building it now...");

        let console_root = Path::new("../console");

        let pnpm_check = Command::new("pnpm").arg("--version").output();

        if pnpm_check.is_err() {
            panic!(
                "pnpm is not installed. Please install pnpm and try again, or build the console manually: cd console && pnpm install && pnpm build"
            );
        }

        println!("cargo:warning=Running pnpm install...");
        let install_status = Command::new("pnpm")
            .arg("install")
            .arg("--frozen-lockfile")
            .current_dir(console_root)
            .status()
            .expect("Failed to run pnpm install");

        if !install_status.success() {
            panic!(
                "Failed to install console dependencies. Try running: cd console && pnpm install"
            );
        }

        println!("cargo:warning=Running pnpm build...");
        let build_status = Command::new("pnpm")
            .arg("build")
            .current_dir(console_root)
            .status()
            .expect("Failed to run pnpm build");

        if !build_status.success() {
            panic!("Failed to build console. Try running: cd console && pnpm build");
        }

        println!("cargo:warning=Console built successfully!");
    }

    if !console_dir.exists() {
        std::fs::create_dir_all(console_dir).expect("Failed to create console build directory");
    }

    println!("cargo:rerun-if-changed=../console/src");
    println!("cargo:rerun-if-changed=../console/package.json");
    println!("cargo:rerun-if-changed=../console/build");
}
