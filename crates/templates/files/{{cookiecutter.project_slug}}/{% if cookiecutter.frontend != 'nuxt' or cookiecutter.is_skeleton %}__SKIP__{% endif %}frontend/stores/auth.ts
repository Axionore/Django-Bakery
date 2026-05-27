import { defineStore } from "pinia";

import type {
  AuthenticatedUser,
  LoginResult,
  MfaActivateBeginResult,
  SignupResult,
} from "~/types/allauth";

type Status = "idle" | "loading" | "loaded" | "error";

/**
 * Auth store — single source of truth for the SPA's auth state.
 *
 * Note the conspicuous absence of any `localStorage` access. All persistence
 * comes from Django's `sessionid` cookie (HttpOnly, set by allauth headless).
 * The SPA cannot read the cookie; it asks the backend "am I logged in?" via
 * `/auth/session` on first paint and trusts what it gets back.
 */
export const useAuthStore = defineStore("auth", {
  state: () => ({
    user: null as AuthenticatedUser | null,
    status: "idle" as Status,
    hydrated: false,
  }),

  actions: {
    async refresh() {
      const auth = useAuth();
      this.status = "loading";
      try {
        const sess = await auth.session();
        this.user = sess.data?.user ?? null;
        this.status = "loaded";
      } catch {
        this.status = "error";
      } finally {
        this.hydrated = true;
      }
    },

    async login(email: string, password: string): Promise<LoginResult> {
      const auth = useAuth();
      const result = await auth.login(email, password);
      if (result.kind === "ok") this.user = result.user;
      return result;
    },

    async signup(email: string, password: string): Promise<SignupResult> {
      const auth = useAuth();
      const result = await auth.signup(email, password);
      if (result.kind === "logged_in") this.user = result.user;
      return result;
    },

    async mfaAuthenticate(code: string): Promise<LoginResult> {
      const auth = useAuth();
      const result = await auth.mfaAuthenticate(code);
      if (result.kind === "ok") this.user = result.user;
      return result;
    },

    mfaActivateBegin(): Promise<MfaActivateBeginResult> {
      return useAuth().mfaActivateBegin();
    },

    mfaActivateConfirm(
      code: string,
    ): Promise<{ kind: "ok" } | { kind: "invalid"; status: number }> {
      return useAuth().mfaActivateConfirm(code);
    },

    recoveryCodes(): Promise<{ codes: string[]; unused: number } | null> {
      return useAuth().recoveryCodes();
    },

    async logout() {
      const auth = useAuth();
      await auth.logout();
      this.user = null;
    },
  },
});
