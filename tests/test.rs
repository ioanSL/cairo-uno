use std::path::PathBuf;

use cairo_lang_compiler::db::RootDatabase;
use cairo_felt::Felt;
use cairo_lang_compiler::diagnostics::check_and_eprint_diagnostics;
use cairo_lang_compiler::project::setup_project;
use cairo_lang_filesystem::ids::CrateId;
use cairo_lang_runner::{RunResultValue, SierraCasmRunner};
use cairo_lang_sierra_generator::db::SierraGenGroup;
use cairo_lang_sierra_generator::replace_ids::replace_sierra_ids_in_program;
use test_case::test_case;

fn setup(name: &str) -> (RootDatabase, Vec<CrateId>) {
    let dir = env!("CARGO_MANIFEST_DIR");
    // Pop the "/tests" suffix.
    let mut path = PathBuf::from(dir).parent().unwrap().to_owned();
    path.push("src");
    path.push(format!("{name}.cairo"));

    let mut db = RootDatabase::default();
    let main_crate_ids = setup_project(&mut db, path.as_path()).expect("Project setup failed.");
    assert!(!check_and_eprint_diagnostics(&mut db));
    (db, main_crate_ids)
}

/// Compiles the Cairo code for `name` to a Sierra program.
fn checked_compile_to_sierra(name: &str) -> cairo_lang_sierra::program::Program {
    let (db, main_crate_ids) = setup(name);
    let sierra_program = db.get_sierra_program(main_crate_ids).unwrap();
    replace_sierra_ids_in_program(&db, &sierra_program)
}

#[test_case(
    "add_one",
    &[41].map(Felt::from), None, None =>
    RunResultValue::Success(vec![Felt::from(42)]);
    "add_one"
)]
#[test_case(
    "add_one",
    &[0].map(Felt::from), None, None =>
    RunResultValue::Success(vec![Felt::from(1)]);
    "add_one_to_zero"
)]
#[test_case(
    "power",
    &[2,3].map(Felt::from), None, None =>
    RunResultValue::Success(vec![Felt::from(8)]);
    "power_2_3"
)]
#[test_case(
    "power",
    &[0,10].map(Felt::from), None, None =>
    RunResultValue::Success(vec![Felt::from(0)]);
    "power_0_10"
)]
#[test_case(
    "power",
    &[10,0].map(Felt::from), None, None =>
    RunResultValue::Success(vec![Felt::from(1)]);
    "power_10_0"
)]
#[test_case(
    "safe_division",
    &[12,3].map(Felt::from), None, None =>
    RunResultValue::Success(vec![Felt::from(4)]);
    "safe_division_12_by_3"
)]
#[test_case(
    "safe_division",
    &[13,3].map(Felt::from), None, None =>
    RunResultValue::Success(vec![Felt::from(4)]);
    "safe_division_13_by_3"
)]
fn run_function_test(
    name: &str,
    params: &[Felt],
    available_gas: Option<usize>,
    expected_cost: Option<usize>,
) -> RunResultValue {
    let runner = SierraCasmRunner::new(checked_compile_to_sierra(name), available_gas.is_some())
        .expect("Failed setting up runner.");
    let result = runner
        .run_function(/* find first */ "", params, available_gas)
        .expect("Failed running the function.");
    if let Some(expected_cost) = expected_cost {
        assert_eq!(available_gas.unwrap() - result.gas_counter.unwrap(), Felt::from(expected_cost));
    }
    result.value
}

#[should_panic]
#[test_case(
    "safe_division",
    &[13,0].map(Felt::from), None, None =>
    RunResultValue::Success(vec![Felt::from(0)]);
    "safe_division_13_by_0"
)]
fn run_function_test_panic(
    name: &str,
    params: &[Felt],
    available_gas: Option<usize>,
    expected_cost: Option<usize>,
) -> RunResultValue {
    let runner = SierraCasmRunner::new(checked_compile_to_sierra(name), available_gas.is_some())
        .expect("Failed setting up runner.");
    let result = runner
        .run_function(/* find first */ "", params, available_gas)
        .expect("Failed running the function.");
    if let Some(expected_cost) = expected_cost {
        assert_eq!(available_gas.unwrap() - result.gas_counter.unwrap(), Felt::from(expected_cost));
    }
    result.value
}
