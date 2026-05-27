import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { setActivePinia, createPinia } from "pinia";

import { useAuthStore } from "~/stores/auth";

const fetchMock = vi.fn();

beforeEach(() => {
  setActivePinia(createPinia());
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

describe("useAuthStore.login", () => {
  it("returns kind=ok and stores the user on 200", async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, {
        meta: { is_authenticated: true },
        data: { user: { id: 1, email: "alice@example.test", full_name: "Alice" } },
      }),
    );
    const store = useAuthStore();
    const result = await store.login("alice@example.test", "correct-horse-battery-staple");
    expect(result.kind).toBe("ok");
    expect(store.user?.email).toBe("alice@example.test");
  });

  it("returns kind=mfa_required when the response carries the mfa_authenticate flow", async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(401, {
        meta: { is_authenticated: false },
        data: { flows: [{ id: "mfa_authenticate", is_pending: true }] },
      }),
    );
    const store = useAuthStore();
    const result = await store.login("a@b.test", "x");
    expect(result.kind).toBe("mfa_required");
    expect(store.user).toBeNull();
  });

  it("returns kind=email_verification_required when verify_email flow is present", async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(401, {
        meta: { is_authenticated: false },
        data: { flows: [{ id: "verify_email" }] },
      }),
    );
    const result = await useAuthStore().login("a@b.test", "x");
    expect(result.kind).toBe("email_verification_required");
  });

  it("returns kind=invalid_credentials on bare 401", async () => {
    fetchMock.mockResolvedValueOnce(jsonResponse(401, { meta: { is_authenticated: false } }));
    expect((await useAuthStore().login("a@b.test", "wrong")).kind).toBe("invalid_credentials");
  });

  it("returns kind=rate_limited on 429", async () => {
    fetchMock.mockResolvedValueOnce(jsonResponse(429, {}));
    expect((await useAuthStore().login("a@b.test", "x")).kind).toBe("rate_limited");
  });

  it("sends credentials=include and the CSRF header", async () => {
    fetchMock.mockResolvedValueOnce(jsonResponse(401, { meta: { is_authenticated: false } }));
    await useAuthStore().login("a@b.test", "x");
    const [_url, init] = fetchMock.mock.calls.at(-1) ?? [];
    expect((init as RequestInit).credentials).toBe("include");
    const headers = (init as RequestInit).headers as Record<string, string>;
    expect(headers["X-CSRFToken"]).toBe("abc123");
  });
});

describe("security invariants", () => {
  it("logout clears the local user", async () => {
    fetchMock.mockResolvedValueOnce(jsonResponse(200, {}));
    const store = useAuthStore();
    store.user = { id: 1, email: "a@b.test", full_name: "A" } as never;
    await store.logout();
    expect(store.user).toBeNull();
  });

  it("never writes a session/token/jwt/auth key to localStorage during login", async () => {
    const setItemSpy = vi.spyOn(Storage.prototype, "setItem");
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, {
        meta: { is_authenticated: true },
        data: { user: { id: 1, email: "a@b.test", full_name: "" } },
      }),
    );
    await useAuthStore().login("a@b.test", "x");
    const offending = setItemSpy.mock.calls.filter(([key]) =>
      /session|token|jwt|auth/i.test(String(key)),
    );
    expect(offending).toEqual([]);
    setItemSpy.mockRestore();
  });
});
