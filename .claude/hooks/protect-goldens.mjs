// PreToolUse hook: golden images change only through `cargo xtask bless`.
// Exit code 2 blocks the tool call and shows stderr to the model.
import { readFileSync } from "node:fs";

let input;
try {
  input = JSON.parse(readFileSync(0, "utf8"));
} catch {
  process.exit(0);
}

const path = String(input?.tool_input?.file_path ?? "").replaceAll("\\", "/");
const isGolden = /(^|\/)tests\/golden\/.+\.png$/i.test(path);
const isScratch = /\.(actual|diff)\.png$/i.test(path);

if (isGolden && !isScratch) {
  console.error(
    "Golden images are not edited by hand. Fix the renderer, or if the new output is " +
      "correct run `cargo xtask bless` and explain the pixel change in the commit message.",
  );
  process.exit(2);
}
