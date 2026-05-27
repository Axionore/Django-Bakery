import { defineConfig } from "vitest/config";
import path from "node:path";

// Vitest tests focus on the BROWSER-side `lib/auth/client.ts` and store.
// Server Components (RSC + cookies()) are hard to unit-test without a Next
// runtime — leave those to Playwright e2e tests.
export default defineConfig({
  resolve: {
    alias: {
      "~": path.resolve(__dirname, "."),
      "~/lib": path.resolve(__dirname, "lib"),
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./tests/setup.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "html"],
      exclude: ["**/*.config.*", ".next/**", "app/**", "tests/e2e/**"],
    },
  },
});
