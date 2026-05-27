/**
 * Reads Django's `csrftoken` cookie and produces headers for mutating fetches.
 *
 * Cookie is not HttpOnly (Django requires JS to set the `X-CSRFToken` header)
 * but IS SameSite=Lax. This is the ONLY module that should touch the cookie —
 * keep it that way so we never accidentally introduce a JWT-in-localStorage
 * pattern through copy-paste.
 */

const CSRF_COOKIE = "csrftoken";

export function getCsrfToken(): string | null {
  if (typeof document === "undefined") return null;
  const pairs = document.cookie ? document.cookie.split("; ") : [];
  for (const pair of pairs) {
    const eq = pair.indexOf("=");
    if (eq < 0) continue;
    if (pair.slice(0, eq) === CSRF_COOKIE) return decodeURIComponent(pair.slice(eq + 1));
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
  try {
    await fetch("/_allauth/browser/v1/auth/session", {
      credentials: "include",
      headers: { Accept: "application/json" },
    });
  } catch {
    /* best-effort */
  }
}
