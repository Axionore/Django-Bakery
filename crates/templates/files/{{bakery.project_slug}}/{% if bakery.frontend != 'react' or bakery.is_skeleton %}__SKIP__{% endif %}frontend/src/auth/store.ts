import { useEffect } from "react";
import { create } from "zustand";

import { authClient } from "~/auth/client";
import type { AuthenticatedUser } from "~/auth/types";

type SessionStatus = "idle" | "loading" | "loaded" | "error";

interface AuthState {
  user: AuthenticatedUser | null;
  status: SessionStatus;
  refresh: () => Promise<void>;
  setUser: (u: AuthenticatedUser | null) => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  status: "idle",
  setUser: (u) => set({ user: u, status: "loaded" }),
  refresh: async () => {
    set({ status: "loading" });
    try {
      const session = await authClient.session();
      set({
        user: session.data?.user ?? null,
        status: "loaded",
      });
    } catch {
      set({ status: "error" });
    }
  },
}));

/** Convenience hook for components that only need `user` + `status`.
 *
 * Selectors are split per-field so each `useAuthStore` call returns a stable
 * primitive/reference. Returning a fresh object literal from a single selector
 * trips React 19's useSyncExternalStore snapshot check, producing
 * "getSnapshot should be cached" warnings and an infinite re-render loop.
 */
export function useAuth(): { user: AuthenticatedUser | null; status: SessionStatus } {
  const user = useAuthStore((s) => s.user);
  const status = useAuthStore((s) => s.status);
  return { user, status };
}

/** Logout action — clears server session, then local store. */
export function useLogout(): () => Promise<void> {
  const setUser = useAuthStore((s) => s.setUser);
  return async () => {
    await authClient.logout();
    setUser(null);
  };
}

/** Mount-time bootstrap — fetches the session once. Idempotent. */
export function useSessionBootstrap(): void {
  const status = useAuthStore((s) => s.status);
  const refresh = useAuthStore((s) => s.refresh);
  useEffect(() => {
    if (status === "idle") void refresh();
  }, [status, refresh]);
}
