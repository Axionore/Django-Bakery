import type { Router } from "vue-router";

import { useAuthStore } from "~/stores/auth";

/**
 * Router-level auth guards.
 *
 * Two annotations a route can declare via `meta`:
 *   - `requiresAuth: true`  → unauthenticated users get bounced to /account/login
 *   - `guest: true`         → already-signed-in users get bounced to /account/profile
 *
 * UX-only: the Django backend enforces every authorization decision. A
 * determined user could disable these guards in devtools, but every request
 * to a guarded backend route still gets 403/302'd by Django.
 */
export function installAuthGuards(router: Router): void {
  router.beforeEach(async (to) => {
    const auth = useAuthStore();
    if (!auth.hydrated) await auth.refresh();

    if (to.meta.requiresAuth && !auth.user) {
      return {
        path: "/account/login",
        query: { next: to.fullPath },
      };
    }
    if (to.meta.guest && auth.user) {
      return { path: "/account/profile" };
    }
    return true;
  });
}
