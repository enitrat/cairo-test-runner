use build_utils::type_generator::generate_types;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("current path: {:?}", std::env::current_dir()?);
    let json_path = "cairo_project/target/dev/sample_project.sierra.json";
    println!("json path: {:?}", json_path);
    let generated_types = generate_types(json_path)?;
    println!("{}", generated_types);
    Ok(())
}
