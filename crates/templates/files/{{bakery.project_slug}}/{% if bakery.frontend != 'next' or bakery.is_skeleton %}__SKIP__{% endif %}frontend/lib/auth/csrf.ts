/**
 * Browser-side CSRF helper.
 *
 * Reads the `csrftoken` cookie (non-HttpOnly by Django design — JS needs it
 * to populate the X-CSRFToken header). For SSR/Server Components, prefer
 * `lib/auth/server.ts` which reads cookies via `next/headers`.
 */

const CSRF_COOKIE = "csrftoken";

export function getCsrfToken(): string | null {
  if (typeof document === "undefined") return null;
  const pairs = document.cookie ? document.cookie.split("; ") : [];
  for (const pair of pairs) {
    const eq = pair.indexOf("=");
    if (eq < 0) continue;
    if (pair.slice(0, eq) === CSRF_COOKIE) {
      return decodeURIComponent(pair.slice(eq + 1));
    }
  }
  return null;
}

export function csrfHeaders(extra: HeadersInit = {}): HeadersInit {
  const token = getCsrfToken();
  const base: Record<string, string> = { "Content-Type": "application/json" };
  if (token) base["X-CSRFToken"] = token;
  return { ...base, ...(extra as Record<string, string>) };
}

export async function ensureCsrfCookie(): Promise<void> {
  if (getCsrfToken()) return;
  await fetch("/_allauth/browser/v1/auth/session", {
    credentials: "include",
    headers: { Accept: "application/json" },
  }).catch(() => {
    /* best-effort */
  });
}
