import { test, expect } from "@playwright/test";

/**
 * E2E smoke tests for the auth flow.
 *
 * CI runs these against `docker compose -f compose.local.yml up` (Django +
 * Postgres + Mailpit) so the Next.js server-side fetches can reach the backend.
 */

test("unauthenticated user hitting /account/profile is redirected (Server Component guard)", async ({ page }) => {
  await page.goto("/account/profile");
  await expect(page).toHaveURL(/\/account\/login/);
});

test("login form surfaces an inline error on bad credentials", async ({ page }) => {
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

test("X-Frame-Options and Permissions-Policy headers ship by default", async ({ page }) => {
  const response = await page.goto("/");
  expect(response).not.toBeNull();
  expect(response!.headers()["x-frame-options"]).toBe("DENY");
  expect(response!.headers()["referrer-policy"]).toBe("same-origin");
  expect(response!.headers()["permissions-policy"]).toContain("camera=()");
});
