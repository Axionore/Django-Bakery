"use client";

import { create } from "zustand";

import { authClient } from "~/lib/auth/client";
import type { AuthenticatedUser } from "~/lib/auth/types";

type Status = "idle" | "loading" | "loaded" | "error";

interface AuthState {
  user: AuthenticatedUser | null;
  status: Status;
  hydrated: boolean;
  setUser: (u: AuthenticatedUser | null) => void;
  refresh: () => Promise<void>;
}

/**
 * Browser-side auth store.
 *
 * Hydration model: the Server Component layout fetches the session via
 * `fetchSessionServer()` and seeds initial user via `<Providers initialUser=…>`,
 * so the first paint already reflects auth state. Subsequent client-side
 * navigations + mutations update the store in place via this module.
 *
 * Notable absence: NO localStorage / sessionStorage persistence. Auth lives
 * in Django's HttpOnly `sessionid` cookie.
 */
export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  status: "idle",
  hydrated: false,
  setUser: (u) => set({ user: u, status: "loaded", hydrated: true }),
  refresh: async () => {
    set({ status: "loading" });
    try {
      const sess = await authClient.session();
      set({
        user: sess.data?.user ?? null,
        status: "loaded",
        hydrated: true,
      });
    } catch {
      set({ status: "error", hydrated: true });
    }
  },
}));

export function useAuth(): { user: AuthenticatedUser | null; status: Status } {
  const user = useAuthStore((s) => s.user);
  const status = useAuthStore((s) => s.status);
  return { user, status };
}

export function useLogout(): () => Promise<void> {
  const setUser = useAuthStore((s) => s.setUser);
  return async () => {
    await authClient.logout();
    setUser(null);
  };
}
