import { csrfHeaders, ensureCsrfCookie } from "~/auth/csrf";
import type {
  AllauthFlow,
  AllauthSession,
  AuthenticatedUser,
  LoginResult,
  MfaActivateBeginResult,
  SignupResult,
} from "~/auth/types";

/**
 * Typed client for django-allauth's headless browser endpoints.
 *
 * Security invariants enforced here (do NOT remove without re-running the threat model):
 *   1. `credentials: "include"` on every fetch — required for session cookies.
 *   2. `X-CSRFToken` on every mutating request via csrfHeaders().
 *   3. Tokens / passwords NEVER stored in localStorage or sessionStorage.
 *   4. Exhaustive Result types so callers can't drop the MFA / email branches.
 */

const BASE = "/_allauth/browser/v1";

async function rawFetch(path: string, init: RequestInit = {}): Promise<Response> {
  return fetch(`${BASE}${path}`, {
    credentials: "include",
    ...init,
    headers: { Accept: "application/json", ...init.headers },
  });
}

function findFlow(body: AllauthSession, id: AllauthFlow["id"]): AllauthFlow | undefined {
  return body.data?.flows?.find((f) => f.id === id);
}

export const authClient = {
  async session(): Promise<AllauthSession> {
    const r = await rawFetch("/auth/session");
    const body = (await r.json()) as AllauthSession;
    body.status = r.status;
    return body;
  },

  async login(email: string, password: string): Promise<LoginResult> {
    await ensureCsrfCookie();
    const r = await rawFetch("/auth/login", {
      method: "POST",
      headers: csrfHeaders(),
      body: JSON.stringify({ email, password }),
    });
    if (r.status === 200) {
      const body = (await r.json()) as AllauthSession;
      const user = body.data?.user;
      if (user) return { kind: "ok", user };
      return { kind: "unknown_error", status: 200 };
    }
    if (r.status === 401) {
      const body = (await r.json()) as AllauthSession;
      const mfa = findFlow(body, "mfa_authenticate");
      if (mfa) return { kind: "mfa_required", flow: mfa };
      const verify = findFlow(body, "verify_email");
      if (verify) return { kind: "email_verification_required" };
      return { kind: "invalid_credentials" };
    }
    if (r.status === 429) return { kind: "rate_limited" };
    return { kind: "unknown_error", status: r.status };
  },

  async signup(email: string, password: string): Promise<SignupResult> {
    await ensureCsrfCookie();
    const r = await rawFetch("/auth/signup", {
      method: "POST",
      headers: csrfHeaders(),
      body: JSON.stringify({ email, password }),
    });
    if (r.status === 200) {
      const body = (await r.json()) as AllauthSession;
      if (findFlow(body, "verify_email")) return { kind: "verification_sent" };
      const user = body.data?.user;
      if (user) return { kind: "logged_in", user };
    }
    if (r.status === 409) return { kind: "duplicate_email" };
    if (r.status === 400) {
      const body = (await r.json().catch(() => ({}))) as {
        errors?: Array<{ param?: string; message?: string }>;
      };
      const fields: Record<string, string[]> = {};
      for (const err of body.errors ?? []) {
        if (!err.param) continue;
        const list = fields[err.param] ?? [];
        if (err.message) list.push(err.message);
        fields[err.param] = list;
      }
      return { kind: "validation_error", fields };
    }
    return { kind: "unknown_error", status: r.status };
  },

  async logout(): Promise<void> {
    await ensureCsrfCookie();
    await rawFetch("/auth/session", { method: "DELETE", headers: csrfHeaders() });
  },

  async mfaAuthenticate(code: string): Promise<LoginResult> {
    await ensureCsrfCookie();
    const r = await rawFetch("/auth/2fa/authenticate", {
      method: "POST",
      headers: csrfHeaders(),
      body: JSON.stringify({ code }),
    });
    if (r.status === 200) {
      const body = (await r.json()) as AllauthSession;
      const user = body.data?.user;
      if (user) return { kind: "ok", user };
    }
    if (r.status === 401) return { kind: "invalid_credentials" };
    if (r.status === 429) return { kind: "rate_limited" };
    return { kind: "unknown_error", status: r.status };
  },

  async mfaActivateBegin(): Promise<MfaActivateBeginResult> {
    await ensureCsrfCookie();
    const r = await rawFetch("/account/authenticators/totp");
    if (r.status === 200) {
      const body = (await r.json()) as { data?: { totp_url?: string; secret?: string } };
      const uri = body.data?.totp_url;
      const secret = body.data?.secret;
      if (uri && secret) return { kind: "ok", uri, secret };
    }
    return { kind: "unknown_error", status: r.status };
  },

  async mfaActivateConfirm(
    code: string,
  ): Promise<{ kind: "ok" } | { kind: "invalid"; status: number }> {
    await ensureCsrfCookie();
    const r = await rawFetch("/account/authenticators/totp", {
      method: "POST",
      headers: csrfHeaders(),
      body: JSON.stringify({ code }),
    });
    if (r.status === 200) return { kind: "ok" };
    return { kind: "invalid", status: r.status };
  },

  async recoveryCodes(): Promise<{ codes: string[]; unused: number } | null> {
    await ensureCsrfCookie();
    const r = await rawFetch("/account/authenticators/recovery-codes");
    if (r.status !== 200) return null;
    const body = (await r.json()) as { data?: { unused_codes?: string[]; total_count?: number } };
    return {
      codes: body.data?.unused_codes ?? [],
      unused: body.data?.unused_codes?.length ?? 0,
    };
  },
};

export type AuthClient = typeof authClient;
export type { AuthenticatedUser };
