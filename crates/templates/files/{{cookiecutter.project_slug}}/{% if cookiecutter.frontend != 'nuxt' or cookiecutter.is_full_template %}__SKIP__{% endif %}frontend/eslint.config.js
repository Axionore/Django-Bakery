import nuxtEslint from "@nuxt/eslint";

// OWASP A03 baseline: no-v-html banned. Nuxt's vue-template-compiler enforces autoescape.
export default nuxtEslint.config({
  rules: {
    "vue/no-v-html": "error",
  },
});
