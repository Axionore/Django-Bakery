import { test, expect } from "@playwright/test";

/**
 * E2E smoke tests for the auth flow.
 *
 * These hit a real Django backend over the Vite dev proxy. CI is expected to
 * have the full Compose stack (django + postgres + mailpit) running before
 * Playwright starts. Locally: `docker compose -f compose.local.yml up` and
 * then `pnpm test:e2e` from the frontend/ dir.
 */

test("unauthenticated user hitting /account/profile is redirected to login", async ({ page }) => {
  await page.goto("/account/profile");
  await expect(page).toHaveURL(/\/account\/login/);
});

test("login form rejects bad credentials with an inline error", async ({ page }) => {
  await page.goto("/account/login");
  await page.getByLabel("Email").fill("nobody@example.test");
  await page.getByLabel("Password").fill("not-the-right-password");
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(page.getByRole("alert")).toContainText(/incorrect/i);
});

test("no session/token/jwt/auth keys land in localStorage during login", async ({ page }) => {
  await page.goto("/account/login");
  await page.getByLabel("Email").fill("nobody@example.test");
  await page.getByLabel("Password").fill("anything");
  await page.getByRole("button", { name: "Sign in" }).click();
  const keys = await page.evaluate(() => Object.keys(window.localStorage));
  expect(keys.filter((k) => /session|token|jwt|auth/i.test(k))).toEqual([]);
});

test("home page renders the project nav", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("link", { name: "About" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Sign in" })).toBeVisible();
});
