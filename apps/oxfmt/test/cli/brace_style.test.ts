import { join } from "node:path";
import { execa } from "execa";
import { describe, expect, it } from "vitest";

const CLI_PATH = join(import.meta.dirname, "..", "..", "dist", "cli.js");

// `vitest.config.ts` pins same-line braces for upstream's tests; drop that to see the real default.
const { OXFMT_TEST_BRACE_STYLE: _pinned, ...env } = process.env;

async function formatStdin(filepath: string, input: string) {
  const { stdout } = await execa("node", [CLI_PATH, "--stdin-filepath", filepath], {
    input,
    env,
    extendEnv: false,
  });
  return stdout;
}

describe("braceStyle", () => {
  it("defaults to next-line braces in JS", async () => {
    expect(await formatStdin("a.ts", "function f(a) { if (a) { b(); } else { c(); } }")).toBe(
      "function f(a)\n{\n  if (a)\n  {\n    b();\n  }\n  else\n  {\n    c();\n  }\n}",
    );
  });

  it("defaults to next-line braces in CSS", async () => {
    expect(await formatStdin("a.css", ".a { color: red; }")).toBe(".a\n{\n  color: red;\n}");
  });
});
