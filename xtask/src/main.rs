use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy)]
struct Target {
    id: &'static str,
    rust_target: &'static str,
    archive: &'static str,
    binary: &'static str,
}

const TARGETS: &[Target] = &[
    Target {
        id: "darwin-amd64",
        rust_target: "x86_64-apple-darwin",
        archive: "t-darwin-amd64.tar.gz",
        binary: "t",
    },
    Target {
        id: "darwin-arm64",
        rust_target: "aarch64-apple-darwin",
        archive: "t-darwin-arm64.tar.gz",
        binary: "t",
    },
    Target {
        id: "linux-amd64",
        rust_target: "x86_64-unknown-linux-gnu",
        archive: "t-linux-amd64.tar.gz",
        binary: "t",
    },
    Target {
        id: "linux-arm64",
        rust_target: "aarch64-unknown-linux-gnu",
        archive: "t-linux-arm64.tar.gz",
        binary: "t",
    },
    Target {
        id: "windows-amd64",
        rust_target: "x86_64-pc-windows-msvc",
        archive: "t-windows-amd64.zip",
        binary: "t.exe",
    },
];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let Some(command) = args.first().cloned() else {
        usage();
        return Ok(());
    };
    args.remove(0);

    match command.as_str() {
        "build-release" => build_release(&args),
        "write-formula" => {
            let repo = repo_dir()?;
            write_checksums_and_formula(&repo)
        }
        _ => {
            usage();
            Err(format!("unknown xtask command: {command}").into())
        }
    }
}

fn build_release(args: &[String]) -> Result<()> {
    let repo = repo_dir()?;
    let target = selected_target(args)?;
    let host = host_target();
    clean_release_outputs(&repo)?;

    if target.id == host.id {
        run_command(
            Command::new("cargo")
                .args(["build", "--release"])
                .current_dir(&repo),
        )?;
    } else {
        run_command(
            Command::new("cargo")
                .args(["build", "--release", "--target", target.rust_target])
                .current_dir(&repo),
        )?;
    }

    package_target(&repo, target, target.id == host.id)?;
    write_checksums_and_formula(&repo)?;
    Ok(())
}

fn selected_target(args: &[String]) -> Result<Target> {
    let mut target_id = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--target-id" => {
                target_id = args.get(i + 1).cloned();
                i += 1;
            }
            value => return Err(format!("unknown build-release option: {value}").into()),
        }
        i += 1;
    }

    if let Some(id) = target_id {
        TARGETS
            .iter()
            .find(|target| target.id == id)
            .copied()
            .ok_or_else(|| format!("unknown target id: {id}").into())
    } else {
        Ok(host_target())
    }
}

fn package_target(repo: &Path, target: Target, is_host: bool) -> Result<()> {
    let dist_dir = repo.join("dist");
    let release_dir = dist_dir.join("release");
    let stage_dir = release_dir.join(target.id);
    fs::remove_dir_all(&stage_dir).ok();
    fs::create_dir_all(&stage_dir)?;

    let built_binary = if is_host {
        repo.join("target").join("release").join(target.binary)
    } else {
        repo.join("target")
            .join(target.rust_target)
            .join("release")
            .join(target.binary)
    };
    let staged_binary = stage_dir.join(target.binary);
    fs::copy(&built_binary, &staged_binary).map_err(|error| {
        format!(
            "failed to copy {} to {}: {error}",
            built_binary.display(),
            staged_binary.display()
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&staged_binary, fs::Permissions::from_mode(0o755))?;
    }

    copy_if_exists(&repo.join("README.md"), &stage_dir.join("README.md"))?;
    copy_if_exists(&repo.join("LICENSE"), &stage_dir.join("LICENSE"))?;

    let archive_path = dist_dir.join(target.archive);
    fs::remove_file(&archive_path).ok();
    if target.archive.ends_with(".zip") {
        archive_zip(&stage_dir, &archive_path)?;
    } else {
        run_command(
            Command::new("tar")
                .args(["-czf", archive_path.to_str().unwrap(), "."])
                .current_dir(&stage_dir),
        )?;
    }
    Ok(())
}

fn clean_release_outputs(repo: &Path) -> Result<()> {
    let dist_dir = repo.join("dist");
    fs::remove_dir_all(dist_dir.join("release")).ok();
    fs::remove_dir_all(dist_dir.join("homebrew")).ok();
    fs::remove_file(dist_dir.join("checksums.txt")).ok();
    for stale in ["t", "t.js", "t.test.js"] {
        fs::remove_file(dist_dir.join(stale)).ok();
    }
    for target in TARGETS {
        fs::remove_file(dist_dir.join(target.archive)).ok();
    }
    Ok(())
}

fn archive_zip(stage_dir: &Path, archive_path: &Path) -> Result<()> {
    if let Ok(status) = Command::new("zip")
        .args(["-qr", archive_path.to_str().unwrap(), "."])
        .current_dir(stage_dir)
        .status()
    {
        if status.success() {
            return Ok(());
        }
    }

    if cfg!(target_os = "windows") {
        run_command(
            Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    "Compress-Archive -Path * -DestinationPath $env:ARCHIVE_PATH -Force",
                ])
                .env("ARCHIVE_PATH", archive_path)
                .current_dir(stage_dir),
        )?;
        return Ok(());
    }

    Err("zip is required to create Windows release archives".into())
}

fn write_checksums_and_formula(repo: &Path) -> Result<()> {
    let dist_dir = repo.join("dist");
    fs::create_dir_all(&dist_dir)?;
    let mut checksum_lines = Vec::new();
    for target in TARGETS {
        let archive_path = dist_dir.join(target.archive);
        if archive_path.exists() {
            checksum_lines.push(format!("{}  {}", sha256(&archive_path)?, target.archive));
        }
    }
    fs::write(
        dist_dir.join("checksums.txt"),
        format!("{}\n", checksum_lines.join("\n")),
    )?;

    let required = ["darwin-amd64", "darwin-arm64", "linux-amd64", "linux-arm64"];
    if required.iter().all(|id| {
        TARGETS
            .iter()
            .any(|target| target.id == *id && dist_dir.join(target.archive).exists())
    }) {
        let formula_dir = dist_dir.join("homebrew").join("Formula");
        fs::create_dir_all(&formula_dir)?;
        fs::write(
            formula_dir.join("translate-cli.rb"),
            homebrew_formula(&dist_dir)?,
        )?;
    }
    Ok(())
}

fn homebrew_formula(dist_dir: &Path) -> Result<String> {
    let version = env::var("RELEASE_VERSION")
        .unwrap_or_else(|_| package_version().unwrap_or_else(|| "0.1.0".to_string()));
    let repository =
        env::var("RELEASE_REPOSITORY").unwrap_or_else(|_| "potato4d/translate-cli".to_string());
    let formula_repository =
        env::var("FORMULA_REPOSITORY").unwrap_or_else(|_| "potato4d/homebrew-tap".to_string());
    let formula_tag =
        env::var("FORMULA_TAG").unwrap_or_else(|_| format!("translate-cli-v{version}"));

    let checksum = |id: &str| -> Result<String> {
        let target = TARGETS.iter().find(|target| target.id == id).unwrap();
        sha256(&dist_dir.join(target.archive))
    };
    let release_url = |archive: &str| -> String {
        format!("https://github.com/{formula_repository}/releases/download/{formula_tag}/{archive}")
    };

    Ok(format!(
        r##"# typed: false
# frozen_string_literal: true

class TranslateCli < Formula
  desc "Translate text through local Agent CLIs"
  homepage "https://github.com/{repository}"
  version "{version}"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "{darwin_amd64_url}"
      sha256 "{darwin_amd64_sha}"
    end
    if Hardware::CPU.arm?
      url "{darwin_arm64_url}"
      sha256 "{darwin_arm64_sha}"
    end
  end

  on_linux do
    if Hardware::CPU.intel? && Hardware::CPU.is_64_bit?
      url "{linux_amd64_url}"
      sha256 "{linux_amd64_sha}"
    end
    if Hardware::CPU.arm? && Hardware::CPU.is_64_bit?
      url "{linux_arm64_url}"
      sha256 "{linux_arm64_sha}"
    end
  end

  def install
    bin.install "t"
  end

  test do
    system "#{{bin}}/t", "--version"
  end
end
"##,
        darwin_amd64_url = release_url("t-darwin-amd64.tar.gz"),
        darwin_amd64_sha = checksum("darwin-amd64")?,
        darwin_arm64_url = release_url("t-darwin-arm64.tar.gz"),
        darwin_arm64_sha = checksum("darwin-arm64")?,
        linux_amd64_url = release_url("t-linux-amd64.tar.gz"),
        linux_amd64_sha = checksum("linux-amd64")?,
        linux_arm64_url = release_url("t-linux-arm64.tar.gz"),
        linux_arm64_sha = checksum("linux-arm64")?,
    ))
}

fn sha256(path: &Path) -> Result<String> {
    for command in ["sha256sum", "shasum"] {
        let output = if command == "shasum" {
            Command::new(command).args(["-a", "256"]).arg(path).output()
        } else {
            Command::new(command).arg(path).output()
        };
        if let Ok(output) = output {
            if output.status.success() {
                let text = String::from_utf8(output.stdout)?;
                if let Some(first) = text.split_whitespace().next() {
                    return Ok(first.to_string());
                }
            }
        }
    }
    Err("sha256sum or shasum is required".into())
}

fn copy_if_exists(source: &Path, destination: &Path) -> Result<()> {
    if source.exists() {
        fs::copy(source, destination)?;
    }
    Ok(())
}

fn run_command(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed with {status}: {command:?}").into())
    }
}

fn repo_dir() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask manifest has no parent".into())
}

fn host_target() -> Target {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    TARGETS
        .iter()
        .find(|target| match (os, arch, target.id) {
            ("macos", "x86_64", "darwin-amd64") => true,
            ("macos", "aarch64", "darwin-arm64") => true,
            ("linux", "x86_64", "linux-amd64") => true,
            ("linux", "aarch64", "linux-arm64") => true,
            ("windows", "x86_64", "windows-amd64") => true,
            _ => false,
        })
        .copied()
        .unwrap_or(TARGETS[0])
}

fn package_version() -> Option<String> {
    let cargo_toml = fs::read_to_string(repo_dir().ok()?.join("Cargo.toml")).ok()?;
    cargo_toml
        .lines()
        .find_map(|line| line.trim().strip_prefix("version = "))
        .and_then(|value| value.trim().trim_matches('"').split('"').next())
        .map(str::to_string)
}

fn usage() {
    eprintln!("Usage:");
    eprintln!("  cargo run -p xtask -- build-release [--target-id <id>]");
    eprintln!("  cargo run -p xtask -- write-formula");
}
