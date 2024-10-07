use std::{env, fs, path::Path, process::Command};
use build_utils::type_generator::generate_types;

fn main() {
    // Change to the cairo_project directory
    let cairo_project_path = Path::new("../../cairo_project");
    env::set_current_dir(&cairo_project_path).expect("Failed to change directory");

    // Run `scarb build` to generate the Sierra file
    let status = Command::new("scarb")
        .arg("build")
        .status()
        .expect("Failed to execute scarb build");

    if !status.success() {
        panic!("scarb build failed");
    }

    // Change back to the root directory
    env::set_current_dir("..").expect("Failed to change back to root directory");

    // Path to the generated Sierra JSON file
    let sierra_json_path = "cairo_project/target/dev/sample_project.sierra.json";

    // Generate Rust types
    let generated_types = generate_types(sierra_json_path)
        .expect("Failed to generate Rust types");

    println!("cargo:warning=Generated types: {:?}", generated_types);

    // Write the generated types to a file in the src directory
    let dest_path = Path::new("crates/test_runner/src").join("generated_types.rs");
    fs::write(&dest_path, generated_types).expect("Failed to write generated types");
    println!("cargo:warning=Wrote types to: {:?}", dest_path);

    println!("cargo:rerun-if-changed={}", sierra_json_path);
    println!("cargo:rerun-if-changed=src/generated_types.rs");
}
