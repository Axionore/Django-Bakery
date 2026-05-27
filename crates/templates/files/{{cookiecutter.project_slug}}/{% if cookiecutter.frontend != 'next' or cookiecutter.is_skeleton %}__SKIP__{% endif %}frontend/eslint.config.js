import next from "eslint-config-next";

/**
 * OWASP A03 (XSS) — ban the raw-HTML React API project-wide. Next's autoescape
 * is the default; this rule prevents a contributor reopening the sink later.
 */
export default [
  ...next(),
  { ignores: ["node_modules", ".next", "out", "lib/api/schema.d.ts"] },
  {
    rules: {
      "react/no-danger": "error",
      "react/no-danger-with-children": "error",
      "no-console": ["warn", { allow: ["warn", "error"] }],
    },
  },
];
