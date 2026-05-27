/**
 * Reads Django's `csrftoken` cookie and produces headers for mutating fetches.
 *
 * The cookie is intentionally NOT HttpOnly (Django requires JS to read it to
 * set the `X-CSRFToken` header). It IS SameSite=Lax so it cannot be exfiltrated
 * cross-site. This composable is the only place that touches the cookie — keep
 * it that way so we never accidentally introduce a JWT-in-localStorage pattern.
 */

const CSRF_COOKIE = "csrftoken";

export function useCsrf() {
  function getCsrfToken(): string | null {
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

  function csrfHeaders(extra: HeadersInit = {}): HeadersInit {
    const token = getCsrfToken();
    const base: Record<string, string> = { "Content-Type": "application/json" };
    if (token) base["X-CSRFToken"] = token;
    return { ...base, ...(extra as Record<string, string>) };
  }

  /**
   * Forces the backend to set the csrftoken cookie if it isn't already. Call
   * once at app bootstrap before any mutating request.
   */
  async function ensureCsrfCookie(): Promise<void> {
    if (getCsrfToken()) return;
    try {
      await $fetch("/_allauth/browser/v1/auth/session", {
        credentials: "include",
        headers: { Accept: "application/json" },
      });
    } catch {
      /* best-effort */
    }
  }

  return { getCsrfToken, csrfHeaders, ensureCsrfCookie };
}
