#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const distDir = path.join(repoDir, "dist");
const releaseDir = path.join(distDir, "release");
const formulaDir = path.join(distDir, "homebrew", "Formula");
const packageJSON = JSON.parse(readFileSync(path.join(repoDir, "package.json"), "utf8"));

const version = process.env.RELEASE_VERSION || packageJSON.version;
const tag = process.env.RELEASE_TAG || `v${version}`;
const repository = process.env.RELEASE_REPOSITORY || "potato4d/translate-cli";

const targets = [
  {
    id: "darwin-amd64",
    bunTarget: "bun-darwin-x64-baseline",
    archive: "t-darwin-amd64.tar.gz",
    binary: "t"
  },
  {
    id: "darwin-arm64",
    bunTarget: "bun-darwin-arm64",
    archive: "t-darwin-arm64.tar.gz",
    binary: "t"
  },
  {
    id: "linux-amd64",
    bunTarget: "bun-linux-x64-baseline",
    archive: "t-linux-amd64.tar.gz",
    binary: "t"
  },
  {
    id: "linux-arm64",
    bunTarget: "bun-linux-arm64",
    archive: "t-linux-arm64.tar.gz",
    binary: "t"
  },
  {
    id: "windows-amd64",
    bunTarget: "bun-windows-x64-baseline",
    archive: "t-windows-amd64.zip",
    binary: "t.exe"
  }
];

rmSync(releaseDir, { recursive: true, force: true });
mkdirSync(releaseDir, { recursive: true });
mkdirSync(formulaDir, { recursive: true });

const archives = new Map();

for (const target of targets) {
  const stageDir = path.join(releaseDir, target.id);
  const archivePath = path.join(distDir, target.archive);
  rmSync(archivePath, { force: true });
  mkdirSync(stageDir, { recursive: true });

  const binaryPath = path.join(stageDir, target.binary);
  run("bun", [
    "build",
    "src/t.ts",
    "--compile",
    `--target=${target.bunTarget}`,
    "--outfile",
    binaryPath
  ], repoDir);

  if (target.binary !== "t.exe") {
    chmodSync(binaryPath, 0o755);
  }
  copyIfExists(path.join(repoDir, "README.md"), path.join(stageDir, "README.md"));
  copyIfExists(path.join(repoDir, "LICENSE"), path.join(stageDir, "LICENSE"));

  if (target.archive.endsWith(".zip")) {
    run("zip", ["-qr", archivePath, "."], stageDir);
  } else {
    run("tar", ["-czf", archivePath, "."], stageDir);
  }

  const sha256 = hashFile(archivePath);
  archives.set(target.id, { ...target, sha256 });
}

writeFileSync(
  path.join(distDir, "checksums.txt"),
  targets
    .map((target) => {
      const archive = archives.get(target.id);
      return `${archive.sha256}  ${target.archive}`;
    })
    .join("\n") + "\n"
);

writeFileSync(
  path.join(formulaDir, "translate-cli.rb"),
  homebrewFormula(archives),
  "utf8"
);

function run(command, args, cwd) {
  const result = spawnSync(command, args, {
    cwd,
    stdio: "inherit"
  });
  if (result.error !== undefined) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed with exit status ${result.status}`);
  }
}

function copyIfExists(source, destination) {
  if (existsSync(source)) {
    copyFileSync(source, destination);
  }
}

function hashFile(file) {
  return createHash("sha256")
    .update(readFileSync(file))
    .digest("hex");
}

function releaseURL(archive) {
  return `https://github.com/${repository}/releases/download/${tag}/${archive}`;
}

function homebrewFormula(archives) {
  const darwinAmd64 = archives.get("darwin-amd64");
  const darwinArm64 = archives.get("darwin-arm64");
  const linuxAmd64 = archives.get("linux-amd64");
  const linuxArm64 = archives.get("linux-arm64");

  return `# typed: false
# frozen_string_literal: true

class TranslateCli < Formula
  desc "Translate text through local Agent CLIs"
  homepage "https://github.com/${repository}"
  version "${version}"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "${releaseURL(darwinAmd64.archive)}"
      sha256 "${darwinAmd64.sha256}"
    end
    if Hardware::CPU.arm?
      url "${releaseURL(darwinArm64.archive)}"
      sha256 "${darwinArm64.sha256}"
    end
  end

  on_linux do
    if Hardware::CPU.intel? && Hardware::CPU.is_64_bit?
      url "${releaseURL(linuxAmd64.archive)}"
      sha256 "${linuxAmd64.sha256}"
    end
    if Hardware::CPU.arm? && Hardware::CPU.is_64_bit?
      url "${releaseURL(linuxArm64.archive)}"
      sha256 "${linuxArm64.sha256}"
    end
  end

  def install
    bin.install "t"
  end

  test do
    system "#{bin}/t", "--version"
  end
end
`;
}
