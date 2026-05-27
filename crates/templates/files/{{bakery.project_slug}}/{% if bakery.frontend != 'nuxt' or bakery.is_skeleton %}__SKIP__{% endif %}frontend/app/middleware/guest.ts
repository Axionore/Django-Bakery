/**
 * `guest` middleware — opt-in via `definePageMeta({ middleware: "guest" })`.
 *
 * Redirects already-signed-in users AWAY from login/signup screens. Prevents
 * the "I'm signed in, why are you showing me the login form?" footgun.
 */
export default defineNuxtRouteMiddleware(async () => {
  if (import.meta.server) return;
  const auth = useAuthStore();
  if (!auth.hydrated) await auth.refresh();
  if (auth.user) return navigateTo("/account/profile");
});
