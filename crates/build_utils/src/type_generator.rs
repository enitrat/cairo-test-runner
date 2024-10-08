use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Clone)]
struct TypeInfo {
    name: String,
    kind: String,
    fields: Vec<String>,
}

fn is_custom_type(name: &str) -> bool {
    name.contains("::") && !name.starts_with("core::") && !name.starts_with("Tuple<")
}

fn extract_type_name(full_name: &str) -> String {
    full_name.split("::").last().unwrap_or(full_name).to_string()
}

pub fn generate_types(json_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(json_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let json: Value = serde_json::from_str(&contents)?;

    let type_declarations = &json["type_declarations"];
    let mut types = HashMap::new();

    for declaration in type_declarations.as_array().unwrap() {
        let id = declaration["id"]["id"].as_u64().unwrap();
        let debug_name = declaration["id"]["debug_name"].as_str().unwrap_or("").to_string();

        if !is_custom_type(&debug_name) {
            continue;
        }

        let type_name = extract_type_name(&debug_name);
        let generic_id = declaration["long_id"]["generic_id"].as_str().unwrap().to_string();

        let mut fields = Vec::new();
        if let Some(generic_args) = declaration["long_id"]["generic_args"].as_array() {
            for arg in generic_args {
                if let Some(type_info) = arg.get("Type") {
                    let field_debug_name = type_info["debug_name"].as_str().unwrap().to_string();
                    fields.push(extract_type_name(&field_debug_name));
                }
            }
        }

        types.insert(id, TypeInfo { name: type_name, kind: generic_id, fields });
    }

    // Generate Rust types
    let mut output = String::new();
    // Push imports
    output.push_str("use starknet_types_core::felt::Felt;\n");
    for type_info in types.values() {
        match type_info.kind.as_str() {
            "Struct" => {
                output.push_str(&format!("#[derive(Debug, PartialEq, Eq)]\npub struct {} {{\n", type_info.name));
                for (i, field) in type_info.fields.iter().enumerate() {
                    output.push_str(&format!("    pub field_{}: {},\n", i, field));
                }
                output.push_str("}\n\n");

                // Add From<Vec<Felt>> implementation
                output.push_str(&format!("impl TryFrom<Vec<Felt>> for {} {{\n", type_info.name));
                output.push_str("    type Error = String;\n");
                output.push_str("    fn try_from(vec: Vec<Felt>) -> Result<Self, String> {\n");
                output.push_str("        Ok(Self {\n");
                for i in 0..type_info.fields.len() {
                    output.push_str(&format!("            field_{}: vec[{}].try_into().unwrap(),\n", i, i));
                }
                output.push_str("        })");
                output.push_str("    }\n");
                output.push_str("}\n\n");
            }
            "Enum" => {
                output.push_str(&format!("#[derive(Debug)]\npub enum {} {{\n", type_info.name));
                for (i, field) in type_info.fields.iter().enumerate() {
                    output.push_str(&format!("    Variant{}({}),\n", i, field));
                }
                output.push_str("}\n\n");
            }
            _ => {
                // For other custom types, we'll use type aliases
                output.push_str(&format!("type {} = {};\n\n", type_info.name, type_info.kind));
            }
        }
    }

    Ok(output)
}
