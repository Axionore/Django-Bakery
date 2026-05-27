<script setup lang="ts">
definePageMeta({ middleware: "auth" });
useSeoMeta({ title: "Set up two-factor auth" });

const uri = ref<string | null>(null);
const secret = ref<string | null>(null);
const code = ref("");
const error = ref<string | null>(null);
const done = ref(false);
const busy = ref(false);
const router = useRouter();
const auth = useAuthStore();

const qrSrc = computed(() =>
  uri.value ? `https://quickchart.io/qr?text=${encodeURIComponent(uri.value)}&size=220` : null,
);

onMounted(async () => {
  const r = await auth.mfaActivateBegin();
  if (r.kind === "ok") {
    uri.value = r.uri;
    secret.value = r.secret;
  } else {
    error.value = "Couldn't start TOTP enrollment.";
  }
});

async function onConfirm(e: Event) {
  e.preventDefault();
  error.value = null;
  if (code.value.length < 6) {
    error.value = "Enter the 6-digit code from your authenticator.";
    return;
  }
  busy.value = true;
  const r = await auth.mfaActivateConfirm(code.value);
  busy.value = false;
  if (r.kind === "ok") {
    done.value = true;
    setTimeout(() => router.push("/account/recovery-codes"), 1200);
  } else {
    error.value = "That code didn't match. Check the time on your device and try again.";
  }
}

function sanitize(v: string) {
  code.value = v.replace(/[^0-9]/g, "").slice(0, 6);
}
</script>

<template>
  <section class="mx-auto max-w-md space-y-5 py-6">
    <h1 class="text-3xl font-semibold">Set up two-factor auth</h1>
    <div
      v-if="done"
      class="rounded-md border border-green-300 bg-green-50 px-3 py-3 text-sm text-green-800 dark:border-green-900 dark:bg-green-950 dark:text-green-100"
    >
      ✔ MFA enabled. Redirecting to your recovery codes…
    </div>
    <template v-else>
      <p class="text-sm text-slate-600 dark:text-slate-400">
        Scan this QR code with an authenticator app (1Password, Authy, Google Authenticator) — or enter
        the secret manually.
      </p>
      <div v-if="uri" class="rounded-md border border-slate-200 bg-slate-50 p-4 dark:border-slate-800 dark:bg-slate-900">
        <img :src="qrSrc!" alt="Two-factor QR code" width="220" height="220" class="block" />
        <p class="mt-3 text-xs text-slate-500 dark:text-slate-400">
          Secret: <code class="rounded bg-slate-100 px-1 py-0.5 dark:bg-slate-800">{{ secret }}</code>
        </p>
      </div>
      <p v-else class="text-sm text-slate-500">Generating…</p>
      <div
        v-if="error"
        role="alert"
        class="rounded-md border border-red-300 bg-red-50 px-3 py-2 text-sm text-red-800 dark:border-red-900 dark:bg-red-950 dark:text-red-200"
      >
        {{ error }}
      </div>
      <form aria-label="Confirm MFA" class="space-y-3" @submit="onConfirm">
        <label class="block text-sm font-medium">Enter the 6-digit code to confirm</label>
        <input
          :value="code"
          inputmode="numeric"
          pattern="[0-9]*"
          autocomplete="one-time-code"
          maxlength="6"
          placeholder="123 456"
          class="block w-full rounded-md border border-slate-300 px-3 py-2 text-center tracking-widest dark:border-slate-700 dark:bg-slate-900"
          @input="(e) => sanitize((e.target as HTMLInputElement).value)"
        />
        <button
          type="submit"
          :disabled="busy"
          class="w-full rounded-md bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-50 dark:bg-slate-100 dark:text-slate-900"
        >
          {{ busy ? "Activating…" : "Activate MFA" }}
        </button>
      </form>
    </template>
  </section>
</template>
