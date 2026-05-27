import vuePlugin from "eslint-plugin-vue";
import vueTsConfig from "@vue/eslint-config-typescript";

// OWASP A03 — XSS defenses. Vue auto-escapes by default; additionally ban
// v-html outright so a future contributor can't open the injection sink by
// accident.
export default [
  ...vuePlugin.configs["flat/recommended"],
  ...vueTsConfig(),
  {
    ignores: ["dist", "node_modules", "src/api/schema.d.ts"],
  },
  {
    rules: {
      "vue/no-v-html": "error",
      "vue/no-v-text-v-html-on-component": "error",
      "no-console": ["warn", { allow: ["warn", "error"] }],
    },
  },
];
