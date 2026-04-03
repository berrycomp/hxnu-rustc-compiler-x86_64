use anyhow::{Context, Result};
use hxnu_target_spec::{
    discover_targets_dir_from_exe, ensure_build_std_flag, ensure_target_arg,
    merged_rust_target_path, PANIC_ABORT,
};
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    if let Err(error) = run() {
        eprintln!("hxnu-cargo: {error:?}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let current_exe = env::current_exe().context("failed to resolve current executable path")?;
    let incoming_args: Vec<String> = env::args().skip(1).collect();
    let cargo_args = prepare_cargo_args(&incoming_args);

    let cargo_bin = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let hxnu_rustc = discover_hxnu_rustc_path(&current_exe)?;
    let targets_dir = discover_targets_dir_from_exe(&current_exe)?;
    let merged_target_path =
        merged_rust_target_path(&targets_dir, env::var_os("RUST_TARGET_PATH"))?;
    let merged_rustflags = merged_rustflags(env::var("RUSTFLAGS").ok());

    let status = Command::new(&cargo_bin)
        .args(&cargo_args)
        .env("RUSTC", &hxnu_rustc)
        .env("RUST_TARGET_PATH", merged_target_path)
        .env("RUSTFLAGS", merged_rustflags)
        .status()
        .with_context(|| format!("failed to execute cargo binary {}", cargo_bin))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn discover_hxnu_rustc_path(current_exe: &Path) -> Result<PathBuf> {
    let exe_dir = current_exe
        .parent()
        .context("failed to resolve executable directory")?;
    let sibling = exe_dir.join("hxnu-rustc");
    if sibling.exists() {
        return Ok(sibling);
    }

    let fallback = exe_dir.join("../hxnu-rustc");
    if fallback.exists() {
        return Ok(fallback);
    }

    Ok(PathBuf::from("hxnu-rustc"))
}

fn prepare_cargo_args(incoming_args: &[String]) -> Vec<String> {
    let mut args = incoming_args.to_vec();
    if should_inject_target(&args) {
        ensure_target_arg(&mut args);
    }
    if should_inject_build_std(&args) {
        ensure_build_std_flag(&mut args);
    }
    args
}

fn should_inject_target(args: &[String]) -> bool {
    compile_subcommand(args).is_some()
}

fn should_inject_build_std(args: &[String]) -> bool {
    compile_subcommand(args).is_some()
}

fn compile_subcommand(args: &[String]) -> Option<&str> {
    let compile_commands = [
        "build", "check", "clippy", "doc", "run", "rustc", "test", "bench",
    ];
    parse_subcommand(args).and_then(|command| {
        if compile_commands.contains(&command) {
            Some(command)
        } else {
            None
        }
    })
}

fn parse_subcommand(args: &[String]) -> Option<&str> {
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        if !arg.starts_with('-') {
            return Some(arg);
        }

        // Global flags that consume the next value.
        let consumes_next = matches!(
            arg,
            "--config"
                | "--manifest-path"
                | "--color"
                | "--jobs"
                | "-j"
                | "--target-dir"
                | "--timings"
                | "--offline"
                | "--frozen"
                | "--locked"
                | "-Z"
        );
        if consumes_next {
            index += 1;
        }
        index += 1;
    }
    None
}

fn merged_rustflags(existing: Option<String>) -> OsString {
    match existing {
        Some(existing) if existing.contains(PANIC_ABORT) => OsString::from(existing),
        Some(existing) => OsString::from(format!("{existing} -C {PANIC_ABORT}")),
        None => OsString::from(format!("-C {PANIC_ABORT}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hxnu_target_spec::TARGET_TRIPLE;

    #[test]
    fn compile_subcommand_adds_target_and_build_std() {
        let args = vec!["build".to_string()];
        let planned = prepare_cargo_args(&args);

        assert!(planned
            .windows(2)
            .any(|w| w[0] == "--target" && w[1] == TARGET_TRIPLE));
        assert!(planned
            .iter()
            .any(|arg| arg == "build-std=core,alloc,compiler_builtins"));
    }

    #[test]
    fn metadata_subcommand_keeps_args() {
        let args = vec![
            "metadata".to_string(),
            "--format-version".to_string(),
            "1".to_string(),
        ];
        let planned = prepare_cargo_args(&args);
        assert_eq!(planned, args);
    }

    #[test]
    fn rustflags_merge_keeps_existing_abort() {
        let merged = merged_rustflags(Some("-C panic=abort -C debuginfo=2".to_string()));
        assert_eq!(merged.to_string_lossy(), "-C panic=abort -C debuginfo=2");
    }

    #[test]
    fn rustflags_merge_appends_abort() {
        let merged = merged_rustflags(Some("-C debuginfo=2".to_string()));
        assert_eq!(merged.to_string_lossy(), "-C debuginfo=2 -C panic=abort");
    }
}
