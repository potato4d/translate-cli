const fs = require("node:fs");
const https = require("node:https");
const os = require("node:os");
const path = require("node:path");

const version = require("../package.json").version;
const platform = process.platform;
const arch = process.arch;

const targets = {
  "darwin-arm64": "t-darwin-arm64.tar.gz",
  "darwin-x64": "t-darwin-amd64.tar.gz",
  "linux-arm64": "t-linux-arm64.tar.gz",
  "linux-x64": "t-linux-amd64.tar.gz",
  "win32-x64": "t-windows-amd64.zip"
};

const target = targets[`${platform}-${arch}`];
if (!target) {
  console.warn(`translate-cli: unsupported platform ${platform}-${arch}; expecting a system t binary`);
  process.exit(0);
}

const url = `https://github.com/potato4d/translate-cli/releases/download/v${version}/${target}`;
const outDir = path.join(__dirname, "..", "vendor", `${platform}-${arch}`);
const archive = path.join(os.tmpdir(), target);

fs.mkdirSync(outDir, { recursive: true });

download(url, archive)
  .then(() => extract(archive, outDir, target))
  .catch((error) => {
    console.warn(`translate-cli: failed to install bundled binary: ${error.message}`);
    console.warn("translate-cli: install a system t binary or retry after the release asset is available");
  });

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (response) => {
      if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
        file.close();
        fs.rmSync(dest, { force: true });
        download(response.headers.location, dest).then(resolve, reject);
        return;
      }
      if (response.statusCode !== 200) {
        file.close();
        fs.rmSync(dest, { force: true });
        reject(new Error(`download failed with HTTP ${response.statusCode}`));
        return;
      }
      response.pipe(file);
      file.on("finish", () => file.close(resolve));
    }).on("error", reject);
  });
}

function extract(archive, outDir, target) {
  const childProcess = require("node:child_process");
  if (target.endsWith(".zip")) {
    childProcess.execFileSync("unzip", ["-o", archive, "-d", outDir], { stdio: "ignore" });
  } else {
    childProcess.execFileSync("tar", ["-xzf", archive, "-C", outDir], { stdio: "ignore" });
  }

  const exe = platform === "win32" ? "t.exe" : "t";
  const binary = path.join(outDir, exe);
  if (fs.existsSync(binary)) {
    fs.chmodSync(binary, 0o755);
  }
}
