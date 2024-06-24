use std::process::Command;
use std::env;
use std::path::PathBuf;

fn main() {
    // Get the target directory
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let target_dir = PathBuf::from(out_dir);
    let target_dir = target_dir
        .parent().expect("No parent directory")
        .parent().expect("No parent directory")
        .parent().expect("No parent directory");

    println!("Target Dir: {}", target_dir.display());

    // Path to the final WASM file
    let wasm_file = target_dir.join("ic_af.wasm");

    // Ensure the WASM file exists before running the command
    if wasm_file.exists() {
        // Run your command to modify the WASM file
        let status = Command::new("sh")
            .arg("-c")
            .arg(format!("wasi2ic {} {}", wasm_file.display(), wasm_file.display()))
            .status()
            .expect("Failed to execute post-build command");

        if !status.success() {
            panic!("Post-build command failed");
        }
    } else {
        panic!("WASM file not found: {}", wasm_file.display());
    }
}