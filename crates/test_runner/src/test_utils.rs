use anyhow::{anyhow, bail, Result};
use cairo_lang_runner::{Arg, RunResultValue, SierraCasmRunner, StarknetState};
use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::{Function, ProgramArtifact, VersionedProgram};
use log::debug;
use crate::deserialization::Args;
use starknet_types_core::felt::Felt;
use std::fs;
use std::path::Path;
use std::str::FromStr;

const DEFAULT_MAIN_FUNCTION: &str = "::main";
const EXECUTABLE_NAME: &str = "starknet_executable";

pub fn load_and_run_cairo_function(function_name: &str, args: &str) -> Result<Vec<Felt>> {
    debug!("Loading and running Cairo function: {}", function_name);
    let sierra_path = Path::new("../../cairo_project/target/dev/sample_project.sierra.json");
    let sierra_program = fs::read_to_string(sierra_path)?;
    let sierra_program: VersionedProgram = serde_json::from_str(&sierra_program)?;
    let sierra_program = sierra_program.into_v1()?;

    let runner = SierraCasmRunner::new(
        sierra_program.program.clone(),
        Some(Default::default()),
        Default::default(),
        None,
    )?;

    let program_artifact = ProgramArtifact {
        program: sierra_program.program,
        debug_info: sierra_program.debug_info,
    };

    let function = main_function(&runner, &program_artifact, Some(function_name))?;

    let deserialized_args = Args::from_str(args)?;
    let runner_args: Vec<Arg> = deserialized_args.into();

    let result = runner.run_function_with_starknet_context(
        function,
        &runner_args,
        None,
        StarknetState::default(),
    )?;

    match result.value {
        RunResultValue::Success(values) => Ok(values),
        RunResultValue::Panic(values) => anyhow::bail!("Function panicked: {:?}", values),
    }
}

fn main_function<'a>(
    runner: &'a SierraCasmRunner,
    sierra_program: &'a ProgramArtifact,
    name: Option<&str>,
) -> Result<&'a Function> {
    let executables = sierra_program
        .debug_info
        .as_ref()
        .and_then(|di| di.executables.get(EXECUTABLE_NAME))
        .cloned()
        .unwrap_or_default();

    // Prioritize `--function` args. First search among executables, then among all functions.
    if let Some(name) = name {
        let name = format!("::{name}");
        return executables
            .iter()
            .find(|fid| {
                fid.debug_name
                    .as_deref()
                    .map(|debug_name| debug_name.ends_with(&name))
                    .unwrap_or_default()
            })
            .map(|fid| find_function(sierra_program, fid))
            .unwrap_or_else(|| Ok(runner.find_function(&name)?));
    }

    // Then check if executables are unambiguous.
    if executables.len() == 1 {
        return find_function(
            sierra_program,
            executables.first().expect("executables can't be empty"),
        );
    }

    // If executables are ambiguous, bail with error.
    if executables.len() > 1 {
        let names = executables
            .iter()
            .flat_map(|fid| fid.debug_name.clone())
            .map(|name| name.to_string())
            .collect::<Vec<_>>();
        let msg = if names.is_empty() {
            "please only mark a single function as executable or enable debug ids and choose function by name".to_string()
        } else {
            format!(
                "please choose a function to run from the list:\n`{}`",
                names.join("`, `")
            )
        };
        bail!("multiple executable functions found\n{msg}");
    }

    // Finally check default function.
    Ok(runner.find_function(DEFAULT_MAIN_FUNCTION)?)
}

fn find_function<'a>(
    sierra_program: &'a ProgramArtifact,
    fid: &FunctionId,
) -> Result<&'a Function> {
    sierra_program
        .program
        .funcs
        .iter()
        .find(|f| f.id == *fid)
        .ok_or_else(|| anyhow!("Function not found: {}", fid.to_string()))
}
