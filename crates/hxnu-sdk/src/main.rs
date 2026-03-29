use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use flate2::write::GzEncoder;
use flate2::Compression;
use hxnu_target_spec::{
    merged_rust_target_path, target_json_path, BUILD_STD_COMPONENTS, PANIC_ABORT, TARGET_TRIPLE,
};
use std::env;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Builder;

const BUNDLE_NAME: &str = "hxnu-rustc-compiler-x86_64";

#[derive(Parser, Debug)]
#[command(name = "hxnu-sdk", about = "HXNU SDK bundle manager")]
struct Cli {
    #[command(subcommand)]
    command: RootCommand,
}

#[derive(Subcommand, Debug)]
enum RootCommand {
    Bundle(BundleArgs),
}

#[derive(Args, Debug)]
struct BundleArgs {
    #[command(subcommand)]
    command: BundleCommand,
}

#[derive(Subcommand, Debug)]
enum BundleCommand {
    Build(BundleBuildArgs),
    Pack(BundlePackArgs),
    Install(BundleInstallArgs),
}

#[derive(Args, Debug)]
struct BundleBuildArgs {
    #[arg(long)]
    out_dir: Option<PathBuf>,
    #[arg(long)]
    version: Option<String>,
    #[arg(long, value_enum, default_value = "release")]
    profile: BuildProfile,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, ValueEnum)]
enum BuildProfile {
    Debug,
    Release,
}

#[derive(Args, Debug)]
struct BundlePackArgs {
    #[arg(long)]
    bundle_dir: Option<PathBuf>,
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct BundleInstallArgs {
    #[arg(long)]
    prefix: PathBuf,
    #[arg(long)]
    bundle_dir: Option<PathBuf>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("hxnu-sdk: {error:?}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let workspace_root = discover_workspace_root()?;

    match cli.command {
        RootCommand::Bundle(bundle) => match bundle.command {
            BundleCommand::Build(args) => {
                let bundle = build_bundle(&workspace_root, &args)?;
                println!("bundle built: {}", bundle.display());
            }
            BundleCommand::Pack(args) => {
                let archive = pack_bundle(&workspace_root, &args)?;
                println!("bundle archive: {}", archive.display());
            }
            BundleCommand::Install(args) => {
                let install_path = install_bundle(&workspace_root, &args)?;
                println!("bundle installed: {}", install_path.display());
            }
        },
    }

    Ok(())
}

fn discover_workspace_root() -> Result<PathBuf> {
    if let Some(explicit) = env::var_os("HXNU_COMPILER_WORKSPACE_ROOT") {
        return Ok(PathBuf::from(explicit));
    }

    let mut cursor = env::current_exe()
        .context("failed to resolve current executable path")?
        .parent()
        .context("failed to resolve executable parent")?
        .to_path_buf();
    for _ in 0..6 {
        let manifest = cursor.join("Cargo.toml");
        if manifest.exists() && cursor.join("targets").exists() && cursor.join("crates").exists() {
            return Ok(cursor);
        }
        if !cursor.pop() {
            break;
        }
    }

    let fallback = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fallback = fallback.canonicalize().unwrap_or(fallback);
    if fallback.join("Cargo.toml").exists() {
        return Ok(fallback);
    }

    bail!("unable to discover workspace root")
}

fn build_bundle(workspace_root: &Path, args: &BundleBuildArgs) -> Result<PathBuf> {
    let out_dir = args
        .out_dir
        .clone()
        .unwrap_or_else(|| workspace_root.join("build/sdk"));
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create output directory {}", out_dir.display()))?;

    let version = args
        .version
        .clone()
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    let bundle_dir = out_dir.join(format!("{BUNDLE_NAME}-{version}"));
    if bundle_dir.exists() {
        fs::remove_dir_all(&bundle_dir)
            .with_context(|| format!("failed to clean bundle directory {}", bundle_dir.display()))?;
    }

    let profile_dir = match args.profile {
        BuildProfile::Debug => "debug",
        BuildProfile::Release => "release",
    };

    run_workspace_build(workspace_root, args.profile)?;

    let bin_dir = bundle_dir.join("bin");
    let targets_dir = bundle_dir.join("targets");
    let sysroot_dir = bundle_dir.join("sysroot/lib/rustlib").join(TARGET_TRIPLE).join("lib");
    let docs_dir = bundle_dir.join("docs");
    let examples_dir = bundle_dir.join("examples");

    for dir in [&bin_dir, &targets_dir, &sysroot_dir, &docs_dir, &examples_dir] {
        fs::create_dir_all(dir)
            .with_context(|| format!("failed to create bundle directory {}", dir.display()))?;
    }

    copy_file(
        &workspace_root.join("target").join(profile_dir).join("hxnu-rustc"),
        &bin_dir.join("hxnu-rustc"),
    )?;
    copy_file(
        &workspace_root.join("target").join(profile_dir).join("hxnu-cargo"),
        &bin_dir.join("hxnu-cargo"),
    )?;
    copy_file(
        &target_json_path(&workspace_root.join("targets")),
        &targets_dir.join(hxnu_target_spec::TARGET_JSON_FILENAME),
    )?;

    copy_directory(&workspace_root.join("docs"), &docs_dir)?;
    copy_directory(&workspace_root.join("examples"), &examples_dir)?;

    let manifest = bundle_dir.join("SDK-MANIFEST.txt");
    fs::write(
        &manifest,
        format!(
            "name={BUNDLE_NAME}\nversion={version}\ntarget={TARGET_TRIPLE}\nprofile={profile_dir}\n"
        ),
    )
    .with_context(|| format!("failed to write manifest {}", manifest.display()))?;

    build_prebuilt_sysroot(workspace_root, profile_dir, &sysroot_dir)?;
    Ok(bundle_dir)
}

fn run_workspace_build(workspace_root: &Path, profile: BuildProfile) -> Result<()> {
    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--manifest-path")
        .arg(workspace_root.join("Cargo.toml"))
        .arg("-p")
        .arg("hxnu-rustc")
        .arg("-p")
        .arg("hxnu-cargo");
    if matches!(profile, BuildProfile::Release) {
        command.arg("--release");
    }

    let status = command.status().context("failed to run cargo build for SDK binaries")?;
    if !status.success() {
        bail!("cargo build for SDK binaries failed");
    }
    Ok(())
}

fn build_prebuilt_sysroot(workspace_root: &Path, profile_dir: &str, out_lib_dir: &Path) -> Result<()> {
    let hxnu_rustc = workspace_root.join("target").join(profile_dir).join("hxnu-rustc");
    let targets_dir = workspace_root.join("targets");
    let merged_target_path = merged_rust_target_path(&targets_dir, env::var_os("RUST_TARGET_PATH"))?;
    let target_dir = workspace_root.join("target/sdk-sysroot");
    fs::create_dir_all(&target_dir)?;

    let status = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(workspace_root.join("examples/no_std-hello/Cargo.toml"))
        .arg("--target")
        .arg(TARGET_TRIPLE)
        .arg("--release")
        .arg("-Z")
        .arg(format!("build-std={BUILD_STD_COMPONENTS}"))
        .env("RUSTC", &hxnu_rustc)
        .env("RUST_TARGET_PATH", merged_target_path)
        .env("RUSTFLAGS", format!("-C {PANIC_ABORT}"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .status()
        .context("failed to run cargo build for sysroot prebuild")?;

    if !status.success() {
        bail!("sysroot prebuild failed");
    }

    let deps_dir = target_dir.join(TARGET_TRIPLE).join("release/deps");
    copy_matching_lib(&deps_dir, out_lib_dir, "libcore-")?;
    copy_matching_lib(&deps_dir, out_lib_dir, "liballoc-")?;
    copy_matching_lib(&deps_dir, out_lib_dir, "libcompiler_builtins-")?;
    Ok(())
}

fn copy_matching_lib(src_dir: &Path, out_dir: &Path, prefix: &str) -> Result<()> {
    let mut copied = false;
    for entry in fs::read_dir(src_dir)
        .with_context(|| format!("failed to read dependency directory {}", src_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let is_lib = file_name.starts_with(prefix)
            && (file_name.ends_with(".rlib") || file_name.ends_with(".rmeta"));
        if !is_lib {
            continue;
        }

        copied = true;
        copy_file(&path, &out_dir.join(file_name.as_ref()))?;
    }

    if !copied {
        bail!("failed to locate {prefix} artifacts in {}", src_dir.display());
    }
    Ok(())
}

fn pack_bundle(workspace_root: &Path, args: &BundlePackArgs) -> Result<PathBuf> {
    let default_bundle = workspace_root
        .join("build/sdk")
        .join(format!("{BUNDLE_NAME}-{}", env!("CARGO_PKG_VERSION")));
    let bundle_dir = args.bundle_dir.clone().unwrap_or(default_bundle);
    if !bundle_dir.exists() {
        bail!(
            "bundle directory does not exist: {} (run `hxnu-sdk bundle build` first)",
            bundle_dir.display()
        );
    }

    let output = args.output.clone().unwrap_or_else(|| {
        workspace_root
            .join("build")
            .join(format!(
                "{}.tar.gz",
                bundle_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(BUNDLE_NAME)
            ))
    });
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let tar_gz = File::create(&output)
        .with_context(|| format!("failed to create archive {}", output.display()))?;
    let encoder = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(encoder);
    let name = bundle_dir
        .file_name()
        .ok_or_else(|| anyhow!("invalid bundle directory name"))?;
    tar.append_dir_all(name, &bundle_dir)
        .with_context(|| format!("failed to archive {}", bundle_dir.display()))?;
    let encoder = tar.into_inner().context("failed to finalize tar archive")?;
    encoder.finish().context("failed to finalize gzip archive")?;

    Ok(output)
}

fn install_bundle(workspace_root: &Path, args: &BundleInstallArgs) -> Result<PathBuf> {
    let default_bundle = workspace_root
        .join("build/sdk")
        .join(format!("{BUNDLE_NAME}-{}", env!("CARGO_PKG_VERSION")));
    let bundle_dir = args.bundle_dir.clone().unwrap_or(default_bundle);
    if !bundle_dir.exists() {
        bail!(
            "bundle directory does not exist: {} (run `hxnu-sdk bundle build` first)",
            bundle_dir.display()
        );
    }

    fs::create_dir_all(&args.prefix)
        .with_context(|| format!("failed to create prefix {}", args.prefix.display()))?;
    let install_root = args.prefix.join(
        bundle_dir
            .file_name()
            .ok_or_else(|| anyhow!("invalid bundle directory name"))?,
    );
    if install_root.exists() {
        fs::remove_dir_all(&install_root)
            .with_context(|| format!("failed to clear install path {}", install_root.display()))?;
    }

    copy_directory(&bundle_dir, &install_root)?;
    Ok(install_root)
}

fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).with_context(|| format!("failed to create {}", dst.display()))?;
    for entry in fs::read_dir(src).with_context(|| format!("failed to read {}", src.display()))? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            copy_directory(&path, &target)?;
        } else if metadata.is_file() {
            copy_file(&path, &target)?;
        }
    }
    Ok(())
}

fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst).with_context(|| format!("failed to copy {} -> {}", src.display(), dst.display()))?;
    Ok(())
}
