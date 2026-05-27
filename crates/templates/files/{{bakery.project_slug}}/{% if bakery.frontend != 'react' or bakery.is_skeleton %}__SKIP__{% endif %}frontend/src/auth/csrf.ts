/**
 * Reads the Django CSRF token from the `csrftoken` cookie.
 *
 * Per Django + allauth headless, the SPA fetches /_allauth/browser/v1/auth/session
 * once on boot to materialize the cookie, then forwards its value in the
 * `X-CSRFToken` header on every mutating call. The cookie is intentionally NOT
 * HttpOnly (Django needs JS to read it) but IS SameSite=Lax, so a malicious
 * cross-site context cannot exfiltrate it.
 */

const CSRF_COOKIE = "csrftoken";

export function getCsrfToken(): string | null {
  if (typeof document === "undefined") return null;
  // Parse cookies manually — no need to pull in a dep for one cookie.
  const pairs = document.cookie ? document.cookie.split("; ") : [];
  for (const pair of pairs) {
    const eq = pair.indexOf("=");
    if (eq < 0) continue;
    const name = pair.slice(0, eq);
    if (name === CSRF_COOKIE) {
      return decodeURIComponent(pair.slice(eq + 1));
    }
  }
  return null;
}

/**
 * Returns headers including X-CSRFToken when the cookie is present. Use on every
 * mutating fetch (POST/PUT/PATCH/DELETE). Safe to call on GETs — Django ignores
 * the header when the method is safe.
 */
export function csrfHeaders(extra: HeadersInit = {}): HeadersInit {
  const token = getCsrfToken();
  const base: Record<string, string> = { "Content-Type": "application/json" };
  if (token) base["X-CSRFToken"] = token;
  // Merge with caller-supplied headers (caller wins on conflict).
  return { ...base, ...(extra as Record<string, string>) };
}

/**
 * Forces the backend to set the csrftoken cookie if it isn't already. Call once
 * during app bootstrap before any mutating request.
 */
export async function ensureCsrfCookie(): Promise<void> {
  if (getCsrfToken()) return;
  await fetch("/_allauth/browser/v1/auth/session", {
    credentials: "include",
    method: "GET",
    headers: { Accept: "application/json" },
  }).catch(() => {
    /* best-effort; the next mutating call will fail with a clearer error */
  });
}
