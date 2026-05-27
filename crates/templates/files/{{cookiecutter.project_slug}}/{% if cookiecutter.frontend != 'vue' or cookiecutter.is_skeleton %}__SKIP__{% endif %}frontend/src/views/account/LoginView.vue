<script setup lang="ts">
import { ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { z } from "zod";

import { useAuthStore } from "~/stores/auth";

const schema = z.object({
  email: z.email("Enter a valid email"),
  password: z.string().min(1, "Password is required"),
});

const email = ref("");
const password = ref("");
const error = ref<string | null>(null);
const busy = ref(false);

const route = useRoute();
const router = useRouter();
const auth = useAuthStore();

async function onSubmit(e: Event) {
  e.preventDefault();
  error.value = null;
  const parsed = schema.safeParse({ email: email.value, password: password.value });
  if (!parsed.success) {
    error.value = parsed.error.issues[0]?.message ?? "Invalid input";
    return;
  }
  busy.value = true;
  const result = await auth.login(parsed.data.email, parsed.data.password);
  busy.value = false;
  switch (result.kind) {
    case "ok":
      await router.push((route.query.next as string) || "/account/profile");
      return;
    case "mfa_required":
      await router.push("/account/mfa-challenge");
      return;
    case "email_verification_required":
      await router.push("/account/verify-email");
      return;
    case "invalid_credentials":
      error.value = "Incorrect email or password.";
      return;
    case "rate_limited":
      error.value = "Too many attempts. Try again in about 5 minutes.";
      return;
    default:
      error.value = "Something went wrong. Please try again.";
  }
}
</script>

<template>
  <div class="space-y-4">
    <h1 class="text-2xl font-semibold">Sign in</h1>
    <p class="text-sm text-slate-600 dark:text-slate-400">Use your email and password.</p>
    <div
      v-if="error"
      role="alert"
      class="rounded-md border border-red-300 bg-red-50 px-3 py-2 text-sm text-red-800 dark:border-red-900 dark:bg-red-950 dark:text-red-200"
    >
      {{ error }}
    </div>
    <form aria-label="Sign in" class="space-y-3" @submit="onSubmit">
      <label class="block text-sm">
        <span class="font-medium">Email</span>
        <input
          v-model="email"
          type="email"
          autocomplete="email"
          required
          class="mt-1 block w-full rounded-md border border-slate-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-900"
        />
      </label>
      <label class="block text-sm">
        <span class="font-medium">Password</span>
        <input
          v-model="password"
          type="password"
          autocomplete="current-password"
          required
          class="mt-1 block w-full rounded-md border border-slate-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-900"
        />
      </label>
      <button
        type="submit"
        :disabled="busy"
        class="w-full rounded-md bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-50 dark:bg-slate-100 dark:text-slate-900"
      >
        {{ busy ? "Signing in…" : "Sign in" }}
      </button>
    </form>
    <p class="text-sm text-slate-600 dark:text-slate-400">
      New here? <RouterLink to="/account/signup" class="underline">Create an account</RouterLink>
    </p>
  </div>
</template>
