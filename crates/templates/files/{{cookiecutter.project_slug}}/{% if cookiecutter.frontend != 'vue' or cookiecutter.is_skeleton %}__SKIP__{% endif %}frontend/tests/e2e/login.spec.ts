import { test, expect } from "@playwright/test";

/**
 * E2E smoke tests for the auth flow.
 * CI must have `docker compose -f compose.local.yml up` running (Django +
 * Postgres + Mailpit) before Playwright starts.
 */

test("unauthenticated user hitting /account/profile is bounced to login", async ({ page }) => {
  await page.goto("/account/profile");
  await expect(page).toHaveURL(/\/account\/login/);
});

test("login form surfaces an inline error on bad credentials", async ({ page }) => {
  await page.goto("/account/login");
  await page.getByLabel("Email").fill("nobody@example.test");
  await page.getByLabel("Password").fill("not-the-right-password");
  await page.getByRole("button", { name: /sign in/i }).click();
  await expect(page.getByRole("alert")).toContainText(/incorrect/i);
});

test("no session/token/jwt/auth keys land in localStorage during login", async ({ page }) => {
  await page.goto("/account/login");
  await page.getByLabel("Email").fill("nobody@example.test");
  await page.getByLabel("Password").fill("anything");
  await page.getByRole("button", { name: /sign in/i }).click();
  const keys = await page.evaluate(() => Object.keys(window.localStorage));
  expect(keys.filter((k) => /session|token|jwt|auth/i.test(k))).toEqual([]);
});

test("home page renders the project nav", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("link", { name: "About" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Sign in" })).toBeVisible();
});
