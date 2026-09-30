#!/usr/bin/env node
// Base npm package wrapper: locates the platform-specific binary that npm
// installed as an optional dependency and runs it with the user's args.
const { spawnSync } = require("child_process");

function getExePath() {
  const arch = process.arch;
  let os = process.platform;
  let extension = "";
  if (["win32", "cygwin"].includes(process.platform)) {
    os = "windows";
    extension = ".exe";
  }
  const pkg = `openflows-cli-${os}-${arch}`;
  try {
    return require.resolve(`${pkg}/bin/openflows-cli${extension}`);
  } catch (e) {
    throw new Error(
      `Couldn't find the openflows-cli binary in node_modules for ${os}-${arch}. ` +
        `Expected package ${pkg}.`
    );
  }
}

function run() {
  const args = process.argv.slice(2);
  const result = spawnSync(getExePath(), args, { stdio: "inherit" });
  process.exit(result.status ?? 0);
}

run();
