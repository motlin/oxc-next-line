import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["./test/**/*.test.ts"],
    snapshotFormat: {
      escapeString: false,
      printBasicPrototype: false,
    },
    snapshotSerializers: [],
    testTimeout: 10000,
    // Fork: upstream's tests assert Prettier's same-line braces; `test/cli/brace_style.test.ts` covers next-line.
    env: { OXFMT_TEST_BRACE_STYLE: "same-line" },
  },
});
