use crate::deserialization::Args;
use anyhow::{anyhow, bail, Result};
use cairo_lang_runner::{Arg, RunResultValue, SierraCasmRunner, StarknetState};
use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::{Function, ProgramArtifact, VersionedProgram};
use log::debug;
use starknet_types_core::felt::Felt;
use std::fs;
use std::path::Path;
use std::str::FromStr;

const DEFAULT_MAIN_FUNCTION: &str = "::main";
const EXECUTABLE_NAME: &str = "starknet_executable";

// struct CustomSierraCasmRunner(SierraCasmRunner);

// impl Deref for CustomSierraCasmRunner {
//     type Target = SierraCasmRunner;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for CustomSierraCasmRunner {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// macro_rules! by_id {
//     ($field:ident) => {{
//         let temp: HashMap<_, _> = test_target_raw
//             .sierra_program
//             .program
//             .$field
//             .iter()
//             .map(|f| (f.id.id, f))
//             .collect();

//         temp
//     }};
// }

// impl CustomSierraCasmRunner {
//         /// Runs the vm starting from a function in the context of a given starknet state.
//         pub fn run_function_with_starknet_context(
//             &self,
//             func: &Function,
//             args: &[Arg],
//             available_gas: Option<usize>,
//             starknet_state: StarknetState,
//         ) -> Result<RunResultStarknet, RunnerError> {
//             let initial_gas = self.get_initial_available_gas(func, available_gas)?;
//             let (entry_code, builtins) = self.create_entry_code(func, args, initial_gas)?;
//             let footer = SierraCasmRunner::create_code_footer();
//             let (hints_dict, string_to_hint) =
//                 build_hints_dict(chain!(&entry_code, &self.get_casm_program().instructions));
//             let assembled_program = self.get_casm_program().clone().assemble_ex(&entry_code, &footer);

//             let funcs = by_id!(funcs);
//             let type_declarations = by_id!(type_declarations);

//             // let data: Vec<MaybeRelocatable> = assembled_program.bytecode.iter().map(Felt::from).map(MaybeRelocatable::from).collect();
//             // let program = Program::new(
//             //     builtins,
//             //     data,
//             //     Some(0),
//             //     hints_dict,
//             //     ReferenceManager { references: Vec::new() },
//             //     HashMap::new(),
//             //     vec![],
//             //     None,
//             // ).expect("Failed to create program");

//             let res = CairoRunner::new(&program, LayoutName::all_cairo, false, true)
//                 .map_err(CairoRunError::from)
//                 .map_err(Box::new).expect("Failed to create runner");

//             let mut hint_processor = CairoHintProcessor {
//                 runner: Some(self),
//                 starknet_state,
//                 string_to_hint,
//                 run_resources: RunResources::default(),
//                 syscalls_used_resources: Default::default(),
//             };

//             let res = casm_run::run_function(assembled_program.bytecode.iter(), builtins, initialize_vm, &mut hint_processor, hints_dict)?;

//             match res {
//                 Ok(run_function_result) => {
//                     let ap = run_function_result.ap;

//                     let return_types = self.generic_id_and_size_from_concrete(&func.signature.ret_types);

//                     let (results_data, gas_counter) = SierraCasmRunner::get_results_data(
//                         &case.test_details.return_types,
//                         &runner.relocated_memory,
//                         ap,
//                     );
//                     assert_eq!(results_data.len(), 1);

//                     let (_, values) = results_data[0].clone();
//                     let value = SierraCasmRunner::handle_main_return_value(
//                         // Here we assume that all test either panic or do not return any value
//                         // This is true for all test right now, but in case it changes
//                         // this logic will need to be updated
//                         Some(0),
//                         values,
//                         &runner.relocated_memory,
//                     );

//                     Ok(RunResultStarknet {
//                         gas_counter,
//                         memory: runner.relocated_memory,
//                         value,
//                         starknet_state: hint_processor.starknet_state,
//                         used_resources: all_used_resources,
//                         profiling_info,
//                     })
//                 }
//                 Err(err) => Err(RunnerError::CairoRunError(err)),
//             };
//         }

//     pub fn handle_main_return_value(
//         inner_type_size: Option<i16>,
//         values: Vec<Felt>,
//         cells: &[Option<Felt>],) -> RunResultValue {
//         println!("Custom handle_main_return_value");
//         // Use inner SierraCasmRunner to handle the return value
//         SierraCasmRunner::handle_main_return_value(inner_type_size, values, cells)
//     }
// }

pub fn load_and_run_cairo_function<T: TryFrom<Vec<Felt>>>(
    function_name: &str,
    args: &str,
) -> Result<T> {
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
        Some(usize::MAX),
        StarknetState::default(),
    )?;

    match result.value {
        RunResultValue::Success(values) => T::try_from(values)
            .map_err(|_| anyhow!("Failed to convert function result to the expected type")),
        RunResultValue::Panic(values) => bail!("Function panicked: {:?}", values),
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
