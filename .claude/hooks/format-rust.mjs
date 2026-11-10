// PostToolUse hook: run rustfmt on any Rust file that was just written or edited.
// Never blocks: a formatting failure (for example a file mid-edit that does not parse)
// is reported by `cargo fmt --check` in the verify step instead.
import { readFileSync, existsSync } from "node:fs";
import { spawnSync } from "node:child_process";

let input;
try {
  input = JSON.parse(readFileSync(0, "utf8"));
} catch {
  process.exit(0);
}

const path = String(input?.tool_input?.file_path ?? "");
if (path.endsWith(".rs") && existsSync(path)) {
  spawnSync("rustfmt", ["--edition", "2021", path], { stdio: "ignore" });
}
