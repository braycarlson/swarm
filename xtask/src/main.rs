use core::fmt::Write as _;
use std::env::consts::{FAMILY, OS};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const BINARY_STEMS: [&str; 3] = ["swarm-cli", "swarm-full", "swarm-gui"];
const DATA_HOME_VARIABLE: &str = "XDG_DATA_HOME";
const DESKTOP_ENTRY_NAME: &str = "swarm.desktop";
const DESKTOP_ICON_COUNT_MAX: u32 = 64;
const DESKTOP_ICON_NAME: &str = "swarm.png";
const DISPLACED_SUFFIX: &str = ".old";
const DISPLACED_SWEEP_COUNT_MAX: u32 = 1_024;
const INSTALL_DIRECTORY_VARIABLE: &str = "SWARM_INSTALL_DIR";
const LAUNCHER_STEM: &str = "swarm-gui";
const LICENSE_VARIABLE: &str = "XWIN_ACCEPT_LICENSE";
const PACKAGE: &str = "swarm";
const TARGET_WINDOWS: &str = "x86_64-pc-windows-msvc";
const USAGE: &str = "\
usage: cargo xtask <task>

tasks:
    build [target]   compile the release binaries for one target triple into
                     target/<triple>/release; target defaults to the host
    dist             build for the host and for Windows, so one checkout
                     produces both deliverables
    install [dir]    build for the host, then copy the binaries into the
                     deployment dir; dir defaults to $SWARM_INSTALL_DIR,
                     else ~/stratus/swarm. On Linux this also installs the
                     desktop entry and the icon theme files, so the window
                     manager can resolve the application icon
";

type Result<T = ()> = core::result::Result<T, Box<dyn core::error::Error>>;

enum Displacement {
    Absent,
    Moved(PathBuf),
}

struct Installation {
    directory: PathBuf,
    target: Target,
}

struct Target {
    triple: String,
}

impl Target {
    fn binary_name(&self, stem: &str) -> String {
        assert_ne!(stem, "");

        if self.is_windows() {
            return format!("{stem}.exe");
        }

        stem.to_owned()
    }

    fn host() -> Result<Self> {
        let output = Command::new(rustc()).arg("-vV").output()?;

        if !output.status.success() {
            return Err(format!("rustc -vV failed: {}", output.status).into());
        }

        let report = String::from_utf8(output.stdout)?;

        let triple = report
            .lines()
            .find_map(|line| line.strip_prefix("host: "))
            .ok_or("rustc -vV named no host triple")?;

        Self::new(triple)
    }

    fn is_windows(&self) -> bool {
        self.triple.contains("-windows-")
    }

    fn new(triple: &str) -> Result<Self> {
        if triple.is_empty() {
            return Err("the target triple is empty".into());
        }

        Ok(Self {
            triple: triple.to_owned(),
        })
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();

    let outcome = match arguments.first().map(String::as_str) {
        Some("build") => build_task(arguments.get(1).map(String::as_str)),
        Some("dist") => dist(),
        Some("install") => install(arguments.get(1).map(String::as_str)),
        _ => {
            eprint!("{USAGE}");

            return ExitCode::FAILURE;
        }
    };

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask: {error}");

            ExitCode::FAILURE
        }
    }
}

fn artifacts_report(artifacts: &[PathBuf]) {
    for artifact in artifacts {
        println!("built {}", artifact.display());
    }
}

fn build(target: &Target) -> Result<Vec<PathBuf>> {
    let root = workspace_root()?;
    let artifacts = release_artifacts(&root, target);
    let mut displaced = Vec::with_capacity(artifacts.len());

    for artifact in &artifacts {
        displaced.push(file_displace(artifact)?);
    }

    let status = cargo_command(target)?
        .args([
            "build",
            "--release",
            "--package",
            PACKAGE,
            "--target",
            &target.triple,
        ])
        .current_dir(&root)
        .status()?;

    if !status.success() {
        for (artifact, displacement) in artifacts.iter().zip(&displaced) {
            file_restore(displacement, artifact)?;
        }

        return Err(format!("cargo build --release for {} failed: {status}", target.triple).into());
    }

    let directory = artifacts
        .first()
        .and_then(|artifact| artifact.parent())
        .ok_or("the release artifacts have no directory")?;

    for stem in BINARY_STEMS {
        displaced_sweep(directory, &target.binary_name(stem))?;
    }

    for artifact in &artifacts {
        assert!(artifact.is_file());
    }

    Ok(artifacts)
}

fn build_task(triple_requested: Option<&str>) -> Result {
    let target = match triple_requested {
        Some(triple) => Target::new(triple)?,
        None => Target::host()?,
    };

    let artifacts = build(&target)?;

    artifacts_report(&artifacts);

    Ok(())
}

fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

fn cargo_command(target: &Target) -> Result<Command> {
    let mut command = Command::new(cargo());

    if target.is_windows() {
        if OS != "windows" {
            license_accepted()?;
            command.arg("xwin");
        }
    }

    Ok(command)
}

fn desktop_caches_refresh(data_home: &Path) {
    let applications = data_home.join("applications");

    let outcome = Command::new("update-desktop-database")
        .arg(&applications)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    let refreshed = outcome.is_ok_and(|status| status.success());

    if !refreshed {
        eprintln!(
            "update-desktop-database did not run; the menu entry appears after the next login",
        );
    }
}

fn desktop_entry_write(data_home: &Path, installation: &Installation) -> Result<PathBuf> {
    let source = workspace_root()?
        .join("assets")
        .join("linux")
        .join(DESKTOP_ENTRY_NAME);

    let template = fs::read_to_string(&source)
        .map_err(|error| format!("{} cannot be read: {error}", source.display()))?;

    let launcher = installation
        .directory
        .join(installation.target.binary_name(LAUNCHER_STEM))
        .display()
        .to_string();

    let mut content = String::with_capacity(template.len() + launcher.len() * 2);

    for line in template.lines() {
        if line.starts_with("Exec=") {
            writeln!(content, "Exec={launcher} %F")?;

            continue;
        }

        if line.starts_with("TryExec=") {
            writeln!(content, "TryExec={launcher}")?;

            continue;
        }

        content.push_str(line);
        content.push('\n');
    }

    let directory = data_home.join("applications");

    fs::create_dir_all(&directory)?;

    let path = directory.join(DESKTOP_ENTRY_NAME);

    fs::write(&path, content)?;

    assert!(path.is_file());

    Ok(path)
}

fn desktop_icons_copy(data_home: &Path) -> Result<u32> {
    let source_root = workspace_root()?
        .join("assets")
        .join("linux")
        .join("icons")
        .join("hicolor");

    let target_root = data_home.join("icons").join("hicolor");
    let mut count: u32 = 0;

    for entry in fs::read_dir(&source_root)? {
        let size_directory = entry?.path();
        let source = size_directory.join("apps").join(DESKTOP_ICON_NAME);

        if !source.is_file() {
            continue;
        }

        if count >= DESKTOP_ICON_COUNT_MAX {
            return Err(format!(
                "{} holds more than {DESKTOP_ICON_COUNT_MAX} icon sizes",
                source_root.display(),
            )
            .into());
        }

        let size_name = size_directory
            .file_name()
            .ok_or("an icon size directory has no name")?;

        let directory = target_root.join(size_name).join("apps");

        fs::create_dir_all(&directory)?;

        let installed = directory.join(DESKTOP_ICON_NAME);
        let byte_count = fs::copy(&source, &installed)?;

        assert!(byte_count >= 1);
        assert!(installed.is_file());

        count += 1;
    }

    if count == 0 {
        return Err(format!("{} holds no icons", source_root.display()).into());
    }

    Ok(count)
}

fn desktop_install(installation: &Installation) -> Result {
    let data_home = directory_from_variable(DATA_HOME_VARIABLE, &[".local", "share"])?;
    let entry = desktop_entry_write(&data_home, installation)?;
    let icon_count = desktop_icons_copy(&data_home)?;

    desktop_caches_refresh(&data_home);

    println!("installed {}", entry.display());
    println!("installed {icon_count} icons under {}", data_home.join("icons").display());

    Ok(())
}

fn directory_from_variable(variable: &str, fallback: &[&str]) -> Result<PathBuf> {
    if let Ok(configured) = std::env::var(variable) {
        if !configured.is_empty() {
            return Ok(PathBuf::from(configured));
        }
    }

    let home_variable = if OS == "windows" {
        "USERPROFILE"
    } else {
        "HOME"
    };

    let home = std::env::var(home_variable).map_err(|error| {
        format!(
            "{variable} and {home_variable} are unset, so no directory can be resolved: \
             {error}",
        )
    })?;

    let mut directory = PathBuf::from(home);

    directory.extend(fallback);

    Ok(directory)
}

fn displaced_sweep(directory: &Path, binary_name: &str) -> Result {
    assert_ne!(binary_name, "");

    let mut examined_count: u32 = 0;

    for entry in fs::read_dir(directory)? {
        examined_count += 1;

        if examined_count > DISPLACED_SWEEP_COUNT_MAX {
            return Err(format!(
                "{} holds more than {DISPLACED_SWEEP_COUNT_MAX} entries, so the sweep stopped",
                directory.display(),
            )
            .into());
        }

        let path = entry?.path();

        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !name.starts_with(binary_name) {
            continue;
        }

        if !name.ends_with(DISPLACED_SUFFIX) {
            continue;
        }

        if let Err(error) = fs::remove_file(&path) {
            eprintln!("could not remove {}: {error}", path.display());
        }
    }

    Ok(())
}

fn dist() -> Result {
    let host = Target::host()?;
    let host_artifacts = build(&host)?;

    artifacts_report(&host_artifacts);

    if host.is_windows() {
        return Ok(());
    }

    let windows_artifacts = build(&Target::new(TARGET_WINDOWS)?)?;

    artifacts_report(&windows_artifacts);

    Ok(())
}

fn file_displace(path: &Path) -> Result<Displacement> {
    if !path.exists() {
        return Ok(Displacement::Absent);
    }

    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let mut displaced_name = path.as_os_str().to_os_string();

    displaced_name.push(format!(".{stamp}{DISPLACED_SUFFIX}"));

    let displaced = PathBuf::from(displaced_name);

    fs::rename(path, &displaced)?;

    assert!(!path.exists());
    assert!(displaced.exists());

    Ok(Displacement::Moved(displaced))
}

fn file_restore(displacement: &Displacement, artifact: &Path) -> Result {
    let Displacement::Moved(displaced) = displacement else {
        return Ok(());
    };

    if artifact.exists() {
        return Ok(());
    }

    fs::rename(displaced, artifact)?;

    assert!(artifact.exists());

    Ok(())
}

fn install(destination: Option<&str>) -> Result {
    let target = Target::host()?;
    let artifacts = build(&target)?;

    assert_eq!(artifacts.len(), BINARY_STEMS.len());

    let directory = match destination {
        Some(path) => PathBuf::from(path),
        None => directory_from_variable(INSTALL_DIRECTORY_VARIABLE, &["stratus", PACKAGE])?,
    };

    fs::create_dir_all(&directory)?;

    for (stem, artifact) in BINARY_STEMS.iter().zip(&artifacts) {
        let name = target.binary_name(stem);
        let installed = directory.join(&name);
        let displacement = file_displace(&installed)?;
        let byte_count = fs::copy(artifact, &installed)?;

        assert!(byte_count >= 1);
        assert!(installed.is_file());

        if let Displacement::Moved(previous) = &displacement {
            assert!(previous.exists());
        }

        displaced_sweep(&directory, &name)?;

        println!("installed {}", installed.display());
    }

    if FAMILY == "unix" {
        if OS != "macos" {
            desktop_install(&Installation { directory, target })?;
        }
    }

    Ok(())
}

fn license_accepted() -> Result {
    if let Ok(accepted) = std::env::var(LICENSE_VARIABLE) {
        if !accepted.is_empty() {
            return Ok(());
        }
    }

    Err(format!(
        "cross-compiling to {TARGET_WINDOWS} downloads the Microsoft CRT and Windows SDK through \
         cargo-xwin. Set {LICENSE_VARIABLE}=1 to accept their license, or run the build on \
         Windows."
    )
    .into())
}

fn release_artifacts(root: &Path, target: &Target) -> Vec<PathBuf> {
    assert!(root.is_absolute());

    let directory = root.join("target").join(&target.triple).join("release");

    let artifacts: Vec<PathBuf> = BINARY_STEMS
        .iter()
        .map(|stem| directory.join(target.binary_name(stem)))
        .collect();

    assert_eq!(artifacts.len(), BINARY_STEMS.len());

    artifacts
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned())
}

fn workspace_root() -> Result<PathBuf> {
    let manifest_directory = Path::new(env!("CARGO_MANIFEST_DIR"));

    let root = manifest_directory
        .parent()
        .ok_or("the xtask manifest directory has no parent")?;

    assert!(root.join("Cargo.toml").is_file());
    assert!(root.join("src").is_dir());

    Ok(root.to_path_buf())
}
