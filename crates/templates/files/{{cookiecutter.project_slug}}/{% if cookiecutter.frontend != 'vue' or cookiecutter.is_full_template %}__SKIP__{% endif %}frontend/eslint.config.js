import vuePlugin from "eslint-plugin-vue";
import vueTsConfig from "@vue/eslint-config-typescript";

// OWASP A03: ban v-html (raw HTML injection sink) outright.
export default [
  ...vuePlugin.configs["flat/recommended"],
  ...vueTsConfig(),
  {
    rules: {
      "vue/no-v-html": "error",
      "vue/no-v-text-v-html-on-component": "error",
    },
  },
];
