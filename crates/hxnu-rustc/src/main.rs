#![feature(rustc_private)]

extern crate rustc_driver;

use anyhow::{Context, Result};
use hxnu_target_spec::{
    discover_targets_dir_from_exe, ensure_panic_abort_codegen, ensure_target_arg,
    load_and_validate_target_spec, merged_rust_target_path, target_json_path,
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
    let target_json = target_json_path(&targets_dir);
    let _summary = load_and_validate_target_spec(&target_json)?;

    let merged = merged_rust_target_path(&targets_dir, env::var_os("RUST_TARGET_PATH"))?;
    env::set_var("RUST_TARGET_PATH", merged);

    let mut rustc_args: Vec<String> = env::args().collect();
    ensure_target_arg(&mut rustc_args);
    ensure_panic_abort_codegen(&mut rustc_args);

    let mut callbacks = DriverCallbacks;
    let exit_code = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&rustc_args, &mut callbacks)
    });
    if exit_code != ExitCode::SUCCESS {
        std::process::exit(1);
    }

    Ok(())
}
