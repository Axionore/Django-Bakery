<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { z } from "zod";

import { useAuthStore } from "~/stores/auth";

const schema = z
  .object({
    email: z.email("Enter a valid email"),
    password: z.string().min(12, "Password must be at least 12 characters"),
    confirm: z.string(),
  })
  .refine((d) => d.password === d.confirm, { message: "Passwords don't match", path: ["confirm"] });

const email = ref("");
const password = ref("");
const confirm = ref("");
const error = ref<string | null>(null);
const fieldErrors = ref<Record<string, string[]>>({});
const busy = ref(false);

const router = useRouter();
const auth = useAuthStore();

async function onSubmit(e: Event) {
  e.preventDefault();
  error.value = null;
  fieldErrors.value = {};
  const parsed = schema.safeParse({
    email: email.value,
    password: password.value,
    confirm: confirm.value,
  });
  if (!parsed.success) {
    error.value = parsed.error.issues[0]?.message ?? "Check your inputs";
    return;
  }
  busy.value = true;
  const result = await auth.signup(parsed.data.email, parsed.data.password);
  busy.value = false;
  switch (result.kind) {
    case "verification_sent":
      await router.push("/account/verify-email");
      return;
    case "logged_in":
      await router.push("/account/profile");
      return;
    case "duplicate_email":
      error.value = "An account already exists for that email.";
      return;
    case "validation_error":
      fieldErrors.value = result.fields;
      error.value = "Check the highlighted fields and try again.";
      return;
    default:
      error.value = "Something went wrong. Please try again.";
  }
}
</script>

<template>
  <div class="space-y-4">
    <h1 class="text-2xl font-semibold">Create your account</h1>
    <div
      v-if="error"
      role="alert"
      class="rounded-md border border-red-300 bg-red-50 px-3 py-2 text-sm text-red-800 dark:border-red-900 dark:bg-red-950 dark:text-red-200"
    >
      {{ '{{' }} error {{ '}}' }}
    </div>
    <form aria-label="Sign up" class="space-y-3" @submit="onSubmit">
      <label class="block text-sm">
        <span class="font-medium">Email</span>
        <input
          v-model="email"
          type="email"
          autocomplete="email"
          required
          class="mt-1 block w-full rounded-md border border-slate-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-900"
        />
        <p v-if="fieldErrors.email" class="mt-1 text-xs text-red-600">{{ '{{' }} fieldErrors.email?.join(" · ") {{ '}}' }}</p>
      </label>
      <label class="block text-sm">
        <span class="font-medium">Password</span>
        <input
          v-model="password"
          type="password"
          autocomplete="new-password"
          required
          class="mt-1 block w-full rounded-md border border-slate-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-900"
        />
        <p class="mt-1 text-xs text-slate-500">At least 12 characters.</p>
        <p v-if="fieldErrors.password" class="mt-1 text-xs text-red-600">{{ '{{' }} fieldErrors.password?.join(" · ") {{ '}}' }}</p>
      </label>
      <label class="block text-sm">
        <span class="font-medium">Confirm password</span>
        <input
          v-model="confirm"
          type="password"
          autocomplete="new-password"
          required
          class="mt-1 block w-full rounded-md border border-slate-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-900"
        />
      </label>
      <button
        type="submit"
        :disabled="busy"
        class="w-full rounded-md bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-50 dark:bg-slate-100 dark:text-slate-900"
      >
        {{ '{{' }} busy ? "Creating…" : "Create account" {{ '}}' }}
      </button>
    </form>
    <p class="text-sm text-slate-600 dark:text-slate-400">
      Already have an account? <RouterLink to="/account/login" class="underline">Sign in</RouterLink>
    </p>
  </div>
</template>
