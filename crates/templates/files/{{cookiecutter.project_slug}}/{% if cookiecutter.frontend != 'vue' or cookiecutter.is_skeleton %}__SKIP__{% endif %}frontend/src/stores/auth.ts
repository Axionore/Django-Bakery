import { defineStore } from "pinia";

import { authClient } from "~/auth/client";
import type {
  AuthenticatedUser,
  LoginResult,
  MfaActivateBeginResult,
  SignupResult,
} from "~/auth/types";

type Status = "idle" | "loading" | "loaded" | "error";

/**
 * Auth store — single source of truth for the SPA's auth state.
 *
 * Note the conspicuous absence of any `localStorage` or `sessionStorage`
 * persistence: Django's `sessionid` cookie (HttpOnly) is the only credential.
 * The SPA cannot read the cookie; it asks the backend "am I logged in?" via
 * `/auth/session` on first paint and trusts what comes back.
 */
export const useAuthStore = defineStore("auth", {
  state: () => ({
    user: null as AuthenticatedUser | null,
    status: "idle" as Status,
    hydrated: false,
  }),

  actions: {
    async refresh() {
      this.status = "loading";
      try {
        const sess = await authClient.session();
        this.user = sess.data?.user ?? null;
        this.status = "loaded";
      } catch {
        this.status = "error";
      } finally {
        this.hydrated = true;
      }
    },

    async login(email: string, password: string): Promise<LoginResult> {
      const result = await authClient.login(email, password);
      if (result.kind === "ok") this.user = result.user;
      return result;
    },

    async signup(email: string, password: string): Promise<SignupResult> {
      const result = await authClient.signup(email, password);
      if (result.kind === "logged_in") this.user = result.user;
      return result;
    },

    async mfaAuthenticate(code: string): Promise<LoginResult> {
      const result = await authClient.mfaAuthenticate(code);
      if (result.kind === "ok") this.user = result.user;
      return result;
    },

    mfaActivateBegin(): Promise<MfaActivateBeginResult> {
      return authClient.mfaActivateBegin();
    },

    mfaActivateConfirm(
      code: string,
    ): Promise<{ kind: "ok" } | { kind: "invalid"; status: number }> {
      return authClient.mfaActivateConfirm(code);
    },

    recoveryCodes(): Promise<{ codes: string[]; unused: number } | null> {
      return authClient.recoveryCodes();
    },

    async logout() {
      await authClient.logout();
      this.user = null;
    },
  },
});
