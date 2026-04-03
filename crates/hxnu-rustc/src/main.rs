#![feature(rustc_private)]

extern crate rustc_driver;

use anyhow::{Context, Result};
use hxnu_target_spec::{
    discover_targets_dir_from_exe, ensure_panic_abort_codegen, ensure_target_arg,
    ensure_unstable_options_flag, extract_target_arg, has_target_arg, is_host_side_compilation,
    load_and_validate_target_spec, merged_rust_target_path, should_inject_rustc_defaults,
    target_id_from_triple, target_json_path, target_json_path_for_triple, TargetId,
};
use std::env;
use std::process::ExitCode;

use rustc_driver::Callbacks;

struct DriverCallbacks;

impl Callbacks for DriverCallbacks {}

fn main() {
    if let Err(error) = run() {
        eprintln!("hxnu-rustc: {error:?}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let current_exe = env::current_exe().context("failed to get current executable path")?;
    let targets_dir = discover_targets_dir_from_exe(&current_exe)?;
    let mut rustc_args: Vec<String> = env::args().collect();
    let should_inject_defaults = should_inject_rustc_defaults(&rustc_args);
    if should_inject_defaults && !is_host_side_compilation(&rustc_args) {
        ensure_target_arg(&mut rustc_args);
        ensure_panic_abort_codegen(&mut rustc_args);
    }
    if should_inject_defaults && has_target_arg(&rustc_args) {
        ensure_unstable_options_flag(&mut rustc_args);
    }

    if let Some(target_triple) = extract_target_arg(&rustc_args) {
        if let Some(target_id) = target_id_from_triple(target_triple) {
            validate_known_target(&targets_dir, target_id)?;
        } else if let Some(target_json) = target_json_path_for_triple(&targets_dir, target_triple) {
            let _summary = load_and_validate_target_spec(&target_json, None)?;
        }
    } else {
        validate_known_target(&targets_dir, TargetId::X86_64)?;
    }

    let merged = merged_rust_target_path(&targets_dir, env::var_os("RUST_TARGET_PATH"))?;
    env::set_var("RUST_TARGET_PATH", merged);

    let mut callbacks = DriverCallbacks;
    let exit_code = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&rustc_args, &mut callbacks)
    });
    if exit_code != ExitCode::SUCCESS {
        std::process::exit(1);
    }

    Ok(())
}

fn validate_known_target(targets_dir: &std::path::Path, target_id: TargetId) -> Result<()> {
    let target_json = target_json_path(targets_dir, target_id);
    let _summary = load_and_validate_target_spec(&target_json, Some(target_id))?;
    Ok(())
}
