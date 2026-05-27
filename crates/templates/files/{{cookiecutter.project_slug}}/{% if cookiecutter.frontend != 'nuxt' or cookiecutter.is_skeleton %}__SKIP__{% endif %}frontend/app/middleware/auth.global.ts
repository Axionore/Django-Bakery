/**
 * Global route middleware — guards every route by default.
 *
 * UX-only: the Django backend remains the authority for every authorization
 * decision. This middleware redirects unauthenticated users to /account/login
 * so they don't see a flash of empty state before the backend kicks them back.
 *
 * Unguarded routes (public) declare themselves with `definePageMeta({ public: true })`.
 */
export default defineNuxtRouteMiddleware(async (to) => {
  // Don't run on the server during initial SSR — the auth state can't be
  // determined without the user's cookie, which isn't present until the request
  // hits Nitro on the client's behalf.
  if (import.meta.server) return;

  const PUBLIC_PREFIXES = ["/account/", "/about", "/"] as const;
  const isPublic = PUBLIC_PREFIXES.some((p) => to.path === p || to.path.startsWith(p + "/"))
    || to.path === "/about"
    || to.path === "/";
  if (isPublic) return;

  const auth = useAuthStore();
  if (!auth.hydrated) await auth.refresh();
  if (!auth.user) {
    return navigateTo({
      path: "/account/login",
      query: { next: to.fullPath },
    });
  }
});
