import js from "@eslint/js";
import tseslint from "@typescript-eslint/eslint-plugin";
import tsparser from "@typescript-eslint/parser";
import reactPlugin from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";

// OWASP A03 (Injection / XSS): ban the raw-HTML React API by default.
export default [
  { ignores: ["dist", "node_modules"] },
  js.configs.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      parser: tsparser,
      parserOptions: { ecmaVersion: "latest", sourceType: "module", ecmaFeatures: { jsx: true } },
    },
    plugins: { "@typescript-eslint": tseslint, react: reactPlugin, "react-hooks": reactHooks },
    rules: {
      ...tseslint.configs.recommended.rules,
      ...reactHooks.configs.recommended.rules,
      "react/no-danger": "error",
      "react/no-danger-with-children": "error",
    },
    settings: { react: { version: "detect" } },
  },
];
