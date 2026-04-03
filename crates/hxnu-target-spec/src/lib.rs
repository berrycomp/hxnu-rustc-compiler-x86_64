use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub const TARGET_TRIPLE_X86_64: &str = "x86_64-unknown-hxnu";
pub const TARGET_TRIPLE_AARCH64: &str = "aarch64-unknown-hxnu";
pub const TARGET_TRIPLE_POWERPC64LE: &str = "powerpc64le-unknown-hxnu";
pub const TARGET_TRIPLE_POWERPC64: &str = "powerpc64-unknown-hxnu";

pub const TARGET_JSON_X86_64: &str = "x86_64-unknown-hxnu.json";
pub const TARGET_JSON_AARCH64: &str = "aarch64-unknown-hxnu.json";
pub const TARGET_JSON_POWERPC64LE: &str = "powerpc64le-unknown-hxnu.json";
pub const TARGET_JSON_POWERPC64: &str = "powerpc64-unknown-hxnu.json";

pub const TARGET_TRIPLE: &str = TARGET_TRIPLE_X86_64;
pub const TARGET_JSON_FILENAME: &str = TARGET_JSON_X86_64;

pub const BUILD_STD_COMPONENTS: &str = "core,alloc,compiler_builtins";
pub const BUILD_STD_FLAG: &str = "-Z";
pub const BUILD_STD_VALUE: &str = "build-std=core,alloc,compiler_builtins";
pub const PANIC_ABORT: &str = "panic=abort";
pub const UNSTABLE_OPTIONS: &str = "unstable-options";

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TargetId {
    X86_64,
    Aarch64,
    Powerpc64Le,
    Powerpc64,
}

impl TargetId {
    pub const fn as_triple(self) -> &'static str {
        match self {
            Self::X86_64 => TARGET_TRIPLE_X86_64,
            Self::Aarch64 => TARGET_TRIPLE_AARCH64,
            Self::Powerpc64Le => TARGET_TRIPLE_POWERPC64LE,
            Self::Powerpc64 => TARGET_TRIPLE_POWERPC64,
        }
    }

    pub const fn json_filename(self) -> &'static str {
        match self {
            Self::X86_64 => TARGET_JSON_X86_64,
            Self::Aarch64 => TARGET_JSON_AARCH64,
            Self::Powerpc64Le => TARGET_JSON_POWERPC64LE,
            Self::Powerpc64 => TARGET_JSON_POWERPC64,
        }
    }

    pub const fn expected_arch(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
            Self::Powerpc64Le | Self::Powerpc64 => "powerpc64",
        }
    }

    pub const fn expected_endian(self) -> &'static str {
        match self {
            Self::X86_64 | Self::Aarch64 | Self::Powerpc64Le => "little",
            Self::Powerpc64 => "big",
        }
    }

    pub const fn expected_llvm_prefix(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
            Self::Powerpc64Le => "powerpc64le",
            Self::Powerpc64 => "powerpc64",
        }
    }
}

pub const SUPPORTED_TARGETS: [TargetId; 4] = [
    TargetId::X86_64,
    TargetId::Aarch64,
    TargetId::Powerpc64Le,
    TargetId::Powerpc64,
];

#[derive(Debug, Clone)]
pub struct TargetSpecSummary {
    pub target_id: Option<TargetId>,
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

pub fn default_target_id() -> TargetId {
    TargetId::X86_64
}

pub fn supported_targets() -> &'static [TargetId] {
    &SUPPORTED_TARGETS
}

pub fn target_id_from_triple(triple: &str) -> Option<TargetId> {
    SUPPORTED_TARGETS
        .iter()
        .copied()
        .find(|target| target.as_triple() == triple)
}

pub fn target_json_filename_for_triple(triple: &str) -> Option<&'static str> {
    target_id_from_triple(triple).map(TargetId::json_filename)
}

pub fn extract_target_arg(args: &[String]) -> Option<&str> {
    let mut index = 0usize;
    while index < args.len() {
        let arg = args[index].as_str();
        if arg == "--target" {
            return args.get(index + 1).map(String::as_str);
        }
        if let Some(value) = arg.strip_prefix("--target=") {
            return Some(value);
        }
        index += 1;
    }
    None
}

pub fn has_target_arg(args: &[String]) -> bool {
    extract_target_arg(args).is_some()
}

pub fn selected_target_id(args: &[String]) -> Option<TargetId> {
    extract_target_arg(args).and_then(target_id_from_triple)
}

pub fn selected_target_or_default(args: &[String]) -> TargetId {
    selected_target_id(args).unwrap_or(default_target_id())
}

pub fn ensure_target_arg(args: &mut Vec<String>) {
    if has_target_arg(args) {
        return;
    }

    args.push("--target".to_string());
    args.push(default_target_id().as_triple().to_string());
}

pub fn has_panic_abort_codegen(args: &[String]) -> bool {
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        if arg == "-C" {
            if iter
                .peek()
                .is_some_and(|value| value.as_str() == PANIC_ABORT)
            {
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
        if has_supported_target_specs(&candidate) {
            return Ok(candidate);
        }
    }

    bail!(
        "failed to locate targets directory near executable {}; expected {}",
        exe.display(),
        expected_target_spec_list()
    )
}

pub fn has_supported_target_specs(targets_dir: &Path) -> bool {
    SUPPORTED_TARGETS
        .iter()
        .all(|target| targets_dir.join(target.json_filename()).exists())
}

pub fn expected_target_spec_list() -> String {
    SUPPORTED_TARGETS
        .iter()
        .map(|target| target.json_filename())
        .collect::<Vec<_>>()
        .join(",")
}

pub fn target_json_path(targets_dir: &Path, target: TargetId) -> PathBuf {
    targets_dir.join(target.json_filename())
}

pub fn target_json_path_for_triple(targets_dir: &Path, triple: &str) -> Option<PathBuf> {
    target_id_from_triple(triple).map(|target| target_json_path(targets_dir, target))
}

pub fn merged_rust_target_path(targets_dir: &Path, existing: Option<OsString>) -> Result<OsString> {
    let mut paths = vec![targets_dir.to_path_buf()];
    if let Some(existing) = existing {
        paths.extend(env::split_paths(&existing));
    }

    env::join_paths(paths).context("failed to build merged RUST_TARGET_PATH")
}

pub fn load_and_validate_target_spec(
    path: &Path,
    expected_target: Option<TargetId>,
) -> Result<TargetSpecSummary> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read target spec {}", path.display()))?;
    let spec: RawTargetSpec =
        serde_json::from_str(&raw).with_context(|| format!("invalid json: {}", path.display()))?;

    let pointer_width = spec.target_pointer_width.as_string();
    if pointer_width != "64" {
        bail!(
            "invalid target-pointer-width {}, expected 64",
            pointer_width
        );
    }

    if let Some(target) = expected_target {
        if spec.arch != target.expected_arch() {
            bail!(
                "invalid arch {}, expected {}",
                spec.arch,
                target.expected_arch()
            );
        }
        if spec.target_endian != target.expected_endian() {
            bail!(
                "invalid target-endian {}, expected {}",
                spec.target_endian,
                target.expected_endian()
            );
        }
        if !spec.llvm_target.starts_with(target.expected_llvm_prefix()) {
            bail!(
                "invalid llvm-target {}, expected {}-compatible target",
                spec.llvm_target,
                target.expected_llvm_prefix()
            );
        }
    } else {
        if spec.arch.is_empty() {
            bail!("invalid arch: empty");
        }
        if spec.target_endian != "little" && spec.target_endian != "big" {
            bail!(
                "invalid target-endian {}, expected little or big",
                spec.target_endian
            );
        }
    }

    Ok(TargetSpecSummary {
        target_id: expected_target,
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
            "-V" | "--version"
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
        if args[index] == "--crate-type" && args.get(index + 1).is_some() {
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

    !has_print_query
        && total_crate_type_count > 0
        && proc_macro_crate_type_count == total_crate_type_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn adds_default_target_when_missing() {
        let mut args = vec!["rustc".into(), "main.rs".into()];
        ensure_target_arg(&mut args);
        assert!(has_target_arg(&args));
        assert!(args
            .windows(2)
            .any(|window| { window[0] == "--target" && window[1] == TARGET_TRIPLE_X86_64 }));
    }

    #[test]
    fn keeps_existing_target() {
        let mut args = vec!["rustc".into(), "--target=aarch64-unknown-hxnu".into()];
        ensure_target_arg(&mut args);
        assert_eq!(args.len(), 2);
        assert_eq!(extract_target_arg(&args), Some(TARGET_TRIPLE_AARCH64));
    }

    #[test]
    fn resolves_target_ids() {
        assert_eq!(
            target_id_from_triple(TARGET_TRIPLE_X86_64),
            Some(TargetId::X86_64)
        );
        assert_eq!(
            target_id_from_triple(TARGET_TRIPLE_AARCH64),
            Some(TargetId::Aarch64)
        );
        assert_eq!(
            target_id_from_triple(TARGET_TRIPLE_POWERPC64LE),
            Some(TargetId::Powerpc64Le)
        );
        assert_eq!(
            target_id_from_triple(TARGET_TRIPLE_POWERPC64),
            Some(TargetId::Powerpc64)
        );
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
    fn validates_target_spec_for_expected_target() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(TARGET_JSON_AARCH64);
        std::fs::write(
            &path,
            r#"{
                "arch": "aarch64",
                "llvm-target": "aarch64-unknown-none-elf",
                "target-endian": "little",
                "target-pointer-width": "64"
            }"#,
        )
        .expect("write");

        let summary =
            load_and_validate_target_spec(&path, Some(TargetId::Aarch64)).expect("valid spec");
        assert_eq!(summary.arch, "aarch64");
        assert_eq!(summary.target_pointer_width, "64");
        assert_eq!(summary.target_id, Some(TargetId::Aarch64));
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
