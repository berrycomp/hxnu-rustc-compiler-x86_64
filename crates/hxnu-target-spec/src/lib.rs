use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub const TARGET_TRIPLE: &str = "x86_64-unknown-hxnu";
pub const TARGET_JSON_FILENAME: &str = "x86_64-unknown-hxnu.json";
pub const BUILD_STD_COMPONENTS: &str = "core,alloc,compiler_builtins";
pub const BUILD_STD_FLAG: &str = "-Z";
pub const BUILD_STD_VALUE: &str = "build-std=core,alloc,compiler_builtins";
pub const PANIC_ABORT: &str = "panic=abort";
pub const UNSTABLE_OPTIONS: &str = "unstable-options";

#[derive(Debug, Clone)]
pub struct TargetSpecSummary {
    pub arch: String,
    pub llvm_target: String,
    pub target_endian: String,
    pub target_pointer_width: String,
    pub linker: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawTargetSpec {
    arch: String,
    #[serde(rename = "llvm-target")]
    llvm_target: String,
    #[serde(rename = "target-endian")]
    target_endian: String,
    #[serde(rename = "target-pointer-width")]
    target_pointer_width: PointerWidth,
    linker: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PointerWidth {
    Number(u16),
    Text(String),
}

impl PointerWidth {
    fn as_string(&self) -> String {
        match self {
            Self::Number(value) => value.to_string(),
            Self::Text(value) => value.clone(),
        }
    }
}

pub fn has_target_arg(args: &[String]) -> bool {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--target" {
            return iter.next().is_some();
        }
        if arg.starts_with("--target=") {
            return true;
        }
    }
    false
}

pub fn ensure_target_arg(args: &mut Vec<String>) {
    if has_target_arg(args) {
        return;
    }

    args.push("--target".to_string());
    args.push(TARGET_TRIPLE.to_string());
}

pub fn has_panic_abort_codegen(args: &[String]) -> bool {
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        if arg == "-C" {
            if iter.peek().is_some_and(|value| value.as_str() == PANIC_ABORT) {
                return true;
            }
            continue;
        }

        if arg == "-Cpanic=abort" || arg == "--codegen=panic=abort" {
            return true;
        }

        if let Some(value) = arg.strip_prefix("-C") {
            if value == PANIC_ABORT {
                return true;
            }
        }
    }
    false
}

pub fn ensure_panic_abort_codegen(args: &mut Vec<String>) {
    if has_panic_abort_codegen(args) {
        return;
    }

    args.push("-C".to_string());
    args.push(PANIC_ABORT.to_string());
}

pub fn has_unstable_options_flag(args: &[String]) -> bool {
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        if arg == "-Z" {
            if iter
                .peek()
                .is_some_and(|value| value.as_str() == UNSTABLE_OPTIONS)
            {
                return true;
            }
            continue;
        }
        if arg == "-Zunstable-options" {
            return true;
        }
    }
    false
}

pub fn ensure_unstable_options_flag(args: &mut Vec<String>) {
    if has_unstable_options_flag(args) {
        return;
    }

    args.push("-Z".to_string());
    args.push(UNSTABLE_OPTIONS.to_string());
}

pub fn has_build_std_flag(args: &[String]) -> bool {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == BUILD_STD_FLAG {
            if iter
                .next()
                .is_some_and(|value| value.starts_with("build-std="))
            {
                return true;
            }
            continue;
        }

        if arg.starts_with("-Zbuild-std=") {
            return true;
        }
    }
    false
}

pub fn ensure_build_std_flag(args: &mut Vec<String>) {
    if has_build_std_flag(args) {
        return;
    }

    args.push(BUILD_STD_FLAG.to_string());
    args.push(BUILD_STD_VALUE.to_string());
}

pub fn discover_targets_dir_from_exe(exe: &Path) -> Result<PathBuf> {
    let exe_dir = exe
        .parent()
        .context("failed to detect executable parent directory")?;

    let candidates = [
        exe_dir.join("../targets"),
        exe_dir.join("../../targets"),
        exe_dir.join("targets"),
    ];

    for candidate in candidates {
        let target_json = candidate.join(TARGET_JSON_FILENAME);
        if target_json.exists() {
            return Ok(candidate);
        }
    }

    bail!(
        "failed to locate targets directory near executable {}; expected {}",
        exe.display(),
        TARGET_JSON_FILENAME
    )
}

pub fn target_json_path(targets_dir: &Path) -> PathBuf {
    targets_dir.join(TARGET_JSON_FILENAME)
}

pub fn merged_rust_target_path(targets_dir: &Path, existing: Option<OsString>) -> Result<OsString> {
    let mut paths = vec![targets_dir.to_path_buf()];
    if let Some(existing) = existing {
        paths.extend(env::split_paths(&existing));
    }

    env::join_paths(paths).context("failed to build merged RUST_TARGET_PATH")
}

pub fn load_and_validate_target_spec(path: &Path) -> Result<TargetSpecSummary> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read target spec {}", path.display()))?;
    let spec: RawTargetSpec =
        serde_json::from_str(&raw).with_context(|| format!("invalid json: {}", path.display()))?;

    if spec.arch != "x86_64" {
        bail!("invalid arch {}, expected x86_64", spec.arch);
    }
    if !spec.llvm_target.starts_with("x86_64") {
        bail!(
            "invalid llvm-target {}, expected x86_64-compatible target",
            spec.llvm_target
        );
    }
    let pointer_width = spec.target_pointer_width.as_string();
    if pointer_width != "64" {
        bail!(
            "invalid target-pointer-width {}, expected 64",
            pointer_width
        );
    }
    if spec.target_endian != "little" {
        bail!("invalid target-endian {}, expected little", spec.target_endian);
    }

    Ok(TargetSpecSummary {
        arch: spec.arch,
        llvm_target: spec.llvm_target,
        target_endian: spec.target_endian,
        target_pointer_width: pointer_width,
        linker: spec.linker,
    })
}

pub fn should_inject_rustc_defaults(args: &[String]) -> bool {
    !args.iter().any(|arg| {
        matches!(
            arg.as_str(),
            "-V"
                | "--version"
                | "-vV"
                | "-h"
                | "--help"
                | "--print"
                | "--explain"
                | "-Whelp"
                | "-Chelp"
                | "-Zhelp"
        )
    })
}

pub fn is_host_side_compilation(args: &[String]) -> bool {
    let mut has_print_query = false;
    let mut total_crate_type_count = 0usize;
    let mut proc_macro_crate_type_count = 0usize;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--print" || args[index].starts_with("--print=") {
            has_print_query = true;
        }
        if args[index] == "--crate-name"
            && args
                .get(index + 1)
                .is_some_and(|value| value.as_str() == "build_script_build")
        {
            return true;
        }
        if args[index] == "--crate-type"
            && args.get(index + 1).is_some()
        {
            total_crate_type_count += 1;
            if args[index + 1] == "proc-macro" {
                proc_macro_crate_type_count += 1;
            }
            index += 1;
        } else if let Some(crate_type) = args[index].strip_prefix("--crate-type=") {
            total_crate_type_count += 1;
            if crate_type == "proc-macro" {
                proc_macro_crate_type_count += 1;
            }
        }
        index += 1;
    }

    !has_print_query && total_crate_type_count > 0 && proc_macro_crate_type_count == total_crate_type_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn adds_target_when_missing() {
        let mut args = vec!["rustc".into(), "main.rs".into()];
        ensure_target_arg(&mut args);
        assert!(has_target_arg(&args));
        assert!(args.windows(2).any(|window| {
            window[0] == "--target" && window[1] == TARGET_TRIPLE
        }));
    }

    #[test]
    fn keeps_existing_target() {
        let mut args = vec!["rustc".into(), "--target=x86_64-custom".into()];
        ensure_target_arg(&mut args);
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn adds_panic_abort_when_missing() {
        let mut args = vec!["rustc".into()];
        ensure_panic_abort_codegen(&mut args);
        assert!(has_panic_abort_codegen(&args));
    }

    #[test]
    fn adds_build_std_when_missing() {
        let mut args = vec!["cargo".into(), "build".into()];
        ensure_build_std_flag(&mut args);
        assert!(has_build_std_flag(&args));
    }

    #[test]
    fn validates_target_spec() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(TARGET_JSON_FILENAME);
        std::fs::write(
            &path,
            r#"{
                "arch": "x86_64",
                "llvm-target": "x86_64-unknown-none",
                "target-endian": "little",
                "target-pointer-width": "64"
            }"#,
        )
        .expect("write");

        let summary = load_and_validate_target_spec(&path).expect("valid spec");
        assert_eq!(summary.arch, "x86_64");
        assert_eq!(summary.target_pointer_width, "64");
    }

    #[test]
    fn skips_injection_for_version_query() {
        let args = vec!["rustc".to_string(), "-vV".to_string()];
        assert!(!should_inject_rustc_defaults(&args));
    }

    #[test]
    fn detects_host_build_script() {
        let args = vec![
            "rustc".to_string(),
            "--crate-name".to_string(),
            "build_script_build".to_string(),
        ];
        assert!(is_host_side_compilation(&args));
    }

    #[test]
    fn detects_host_proc_macro_compile() {
        let args = vec![
            "rustc".to_string(),
            "--crate-type".to_string(),
            "proc-macro".to_string(),
        ];
        assert!(is_host_side_compilation(&args));
    }

    #[test]
    fn ignores_probe_with_proc_macro_crate_type() {
        let args = vec![
            "rustc".to_string(),
            "-".to_string(),
            "--print=file-names".to_string(),
            "--crate-type".to_string(),
            "bin".to_string(),
            "--crate-type".to_string(),
            "proc-macro".to_string(),
        ];
        assert!(!is_host_side_compilation(&args));
    }
}
