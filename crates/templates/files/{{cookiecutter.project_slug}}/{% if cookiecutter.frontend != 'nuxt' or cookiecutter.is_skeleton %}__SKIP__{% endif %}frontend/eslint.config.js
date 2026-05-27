import nuxtEslint from "@nuxt/eslint";

// OWASP A03 — XSS defenses. Vue auto-escapes by default; we additionally ban
// v-html outright and require explicit components for any rich-content render.
export default nuxtEslint.config({
  rules: {
    "vue/no-v-html": "error",
    "vue/no-v-text-v-html-on-component": "error",
    "no-console": ["warn", { allow: ["warn", "error"] }],
  },
});
