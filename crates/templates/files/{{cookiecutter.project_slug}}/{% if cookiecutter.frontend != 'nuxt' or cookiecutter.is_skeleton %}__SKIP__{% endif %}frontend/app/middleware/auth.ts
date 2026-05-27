/**
 * Per-page `auth` middleware — opt-in via `definePageMeta({ middleware: "auth" })`.
 *
 * For pages that must NOT render without a session (profile, mfa-activate, etc).
 * Pairs with `auth.global.ts` which only protects /account/profile + descendants
 * by default; pages outside that prefix opt in explicitly.
 */
export default defineNuxtRouteMiddleware(async (to) => {
  if (import.meta.server) return;
  const auth = useAuthStore();
  if (!auth.hydrated) await auth.refresh();
  if (!auth.user) {
    return navigateTo({
      path: "/account/login",
      query: { next: to.fullPath },
    });
  }
});
