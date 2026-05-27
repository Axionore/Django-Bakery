import { defineConfig, mergeConfig } from "vitest/config";
import viteConfig from "./vite.config";

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      environment: "happy-dom",
      globals: true,
      exclude: ["node_modules", "tests/e2e/**", "dist"],
      coverage: {
        provider: "v8",
        reporter: ["text", "html"],
        exclude: ["**/*.config.*", "dist/**", "tests/e2e/**"],
      },
    },
  }),
);
