<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";

import { useAuthStore } from "~/stores/auth";

const code = ref("");
const error = ref<string | null>(null);
const busy = ref(false);
const router = useRouter();
const auth = useAuthStore();

function sanitize(v: string) {
  code.value = v.replace(/[^0-9]/g, "").slice(0, 6);
}

async function onSubmit(e: Event) {
  e.preventDefault();
  error.value = null;
  if (code.value.length < 6) {
    error.value = "Enter the 6-digit code from your authenticator app.";
    return;
  }
  busy.value = true;
  const result = await auth.mfaAuthenticate(code.value);
  busy.value = false;
  switch (result.kind) {
    case "ok":
      await router.push("/account/profile");
      return;
    case "invalid_credentials":
      error.value = "That code didn't match. Try again.";
      return;
    case "rate_limited":
      error.value = "Too many attempts. Wait a few minutes.";
      return;
    default:
      error.value = "Couldn't verify. Try again.";
  }
}
</script>

<template>
  <div class="space-y-4">
    <h1 class="text-2xl font-semibold">Two-factor code</h1>
    <p class="text-sm text-slate-600 dark:text-slate-400">Enter the 6-digit code from your authenticator app.</p>
    <div
      v-if="error"
      role="alert"
      class="rounded-md border border-red-300 bg-red-50 px-3 py-2 text-sm text-red-800 dark:border-red-900 dark:bg-red-950 dark:text-red-200"
    >
      {{ '{{' }} error {{ '}}' }}
    </div>
    <form aria-label="MFA code" class="space-y-3" @submit="onSubmit">
      <input
        :value="code"
        inputmode="numeric"
        pattern="[0-9]*"
        autocomplete="one-time-code"
        maxlength="6"
        placeholder="123456"
        class="block w-full rounded-md border border-slate-300 px-3 py-2 text-center tracking-widest dark:border-slate-700 dark:bg-slate-900"
        @input="(e) => sanitize((e.target as HTMLInputElement).value)"
      />
      <button
        type="submit"
        :disabled="busy"
        class="w-full rounded-md bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-50 dark:bg-slate-100 dark:text-slate-900"
      >
        {{ '{{' }} busy ? "Verifying…" : "Verify" {{ '}}' }}
      </button>
    </form>
  </div>
</template>
