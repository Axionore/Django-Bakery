import next from "eslint-config-next";

// OWASP A03 (XSS): no-danger banned. Next.js auto-escapes JSX by default.
export default [
  ...next(),
  {
    rules: {
      "react/no-danger": "error",
      "react/no-danger-with-children": "error",
    },
  },
];
