use std::env;
use std::process::Command;

fn main() {
    // Only run this build script when compiling in release mode
    if env::var("PROFILE").unwrap() == "release" {
        let out_dir = env::var("OUT_DIR").unwrap();
        let binary_path = format!("{}/../../../pm", out_dir);
        let destination = "/usr/local/bin/pm";

        // Try copying the binary
        let output = Command::new("sudo")
            .arg("cp")
            .arg(&binary_path)
            .arg(destination)
            .output()
            .expect("Failed to copy the binary to /usr/local/bin");

        if !output.status.success() {
            eprintln!(
                "Error copying binary: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        } else {
            println!("Successfully copied binary to {}", destination);
        }
    }
}
