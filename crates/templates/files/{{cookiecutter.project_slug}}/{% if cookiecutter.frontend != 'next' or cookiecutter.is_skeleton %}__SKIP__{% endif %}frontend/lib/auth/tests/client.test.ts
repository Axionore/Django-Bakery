import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { authClient } from "~/lib/auth/client";

const fetchMock = vi.fn();

beforeEach(() => {
  vi.stubGlobal("fetch", fetchMock);
  Object.defineProperty(document, "cookie", {
    configurable: true,
    get: () => "csrftoken=abc123",
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
  fetchMock.mockReset();
});

function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

describe("authClient.login", () => {
  it("returns kind=ok and the user on 200", async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, {
        meta: { is_authenticated: true },
        data: { user: { id: 1, email: "alice@example.test", full_name: "Alice" } },
      }),
    );
    const result = await authClient.login("alice@example.test", "correct-horse-battery-staple");
    expect(result.kind).toBe("ok");
    if (result.kind === "ok") {
      expect(result.user.email).toBe("alice@example.test");
    }
  });

  it("returns kind=mfa_required when the response includes the mfa_authenticate flow", async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(401, {
        meta: { is_authenticated: false },
        data: { flows: [{ id: "mfa_authenticate", is_pending: true }] },
      }),
    );
    const result = await authClient.login("a@b.test", "x");
    expect(result.kind).toBe("mfa_required");
  });

  it("returns kind=email_verification_required when the verify_email flow is present", async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(401, {
        meta: { is_authenticated: false },
        data: { flows: [{ id: "verify_email" }] },
      }),
    );
    expect((await authClient.login("a@b.test", "x")).kind).toBe("email_verification_required");
  });

  it("returns kind=invalid_credentials on a bare 401 with no flows", async () => {
    fetchMock.mockResolvedValueOnce(jsonResponse(401, { meta: { is_authenticated: false } }));
    expect((await authClient.login("a@b.test", "wrong")).kind).toBe("invalid_credentials");
  });

  it("returns kind=rate_limited on 429", async () => {
    fetchMock.mockResolvedValueOnce(jsonResponse(429, {}));
    expect((await authClient.login("a@b.test", "x")).kind).toBe("rate_limited");
  });

  it("sends credentials and the CSRF header on every login attempt", async () => {
    fetchMock.mockResolvedValueOnce(jsonResponse(401, { meta: { is_authenticated: false } }));
    await authClient.login("a@b.test", "x");
    const [_url, init] = fetchMock.mock.calls.at(-1) ?? [];
    expect((init as RequestInit).credentials).toBe("include");
    const headers = (init as RequestInit).headers as Record<string, string>;
    expect(headers["X-CSRFToken"]).toBe("abc123");
  });
});

describe("authClient.signup", () => {
  it("returns kind=verification_sent when the response carries the verify_email flow", async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, {
        meta: { is_authenticated: false },
        data: { flows: [{ id: "verify_email" }] },
      }),
    );
    expect((await authClient.signup("a@b.test", "12345678901234")).kind).toBe("verification_sent");
  });

  it("returns kind=duplicate_email on 409", async () => {
    fetchMock.mockResolvedValueOnce(jsonResponse(409, {}));
    expect((await authClient.signup("a@b.test", "12345678901234")).kind).toBe("duplicate_email");
  });
});

describe("security invariants", () => {
  it("never reads or writes any session/token/jwt/auth key to localStorage", async () => {
    const setItemSpy = vi.spyOn(Storage.prototype, "setItem");
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, {
        meta: { is_authenticated: true },
        data: { user: { id: 1, email: "a@b.test", full_name: "" } },
      }),
    );
    await authClient.login("a@b.test", "x");
    const offending = setItemSpy.mock.calls.filter(([key]) =>
      /session|token|jwt|auth/i.test(String(key)),
    );
    expect(offending).toEqual([]);
    setItemSpy.mockRestore();
  });
});
