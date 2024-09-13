use std::process::Command;
use std::env;

fn main() {
    // Only run this build script when compiling in release mode
    if env::var("PROFILE").unwrap() == "release" {
        // Get the current directory (should be your project root)
        let out_dir = env::var("OUT_DIR").unwrap();

        // Define the binary path (target/release/pm)
        let binary_path = format!("{}/../../../pm", out_dir);
        
        // Define the target destination (usr/local/bin/)
        let destination = "/usr/local/bin/pm";

        // Try copying the binary
        let output = Command::new("sudo")
            .arg("cp")
            .arg(&binary_path)
            .arg(destination)
            .output()
            .expect("Failed to copy the binary to /usr/local/bin");

        // If the copy command failed, print the error
        if !output.status.success() {
            eprintln!("Error copying binary: {}", String::from_utf8_lossy(&output.stderr));
        } else {
            println!("Successfully copied binary to {}", destination);
        }
    }
}
