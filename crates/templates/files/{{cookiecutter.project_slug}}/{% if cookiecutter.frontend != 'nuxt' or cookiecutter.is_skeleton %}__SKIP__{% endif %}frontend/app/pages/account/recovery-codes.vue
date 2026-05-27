<script setup lang="ts">
definePageMeta({ middleware: "auth" });
useSeoMeta({ title: "Recovery codes" });

const auth = useAuthStore();
const codes = ref<string[] | null>(null);

onMounted(async () => {
  const r = await auth.recoveryCodes();
  codes.value = r?.codes ?? [];
});
</script>

<template>
  <section class="mx-auto max-w-md space-y-5 py-6">
    <h1 class="text-3xl font-semibold">Recovery codes</h1>
    <div class="rounded-md border border-amber-300 bg-amber-50 px-3 py-3 text-sm text-amber-900 dark:border-amber-900 dark:bg-amber-950 dark:text-amber-100">
      ⚠ Store these somewhere safe (1Password, a paper wallet). Each code works <strong>once</strong>.
      You can use one to sign in if you lose access to your authenticator app.
    </div>
    <p v-if="codes === null" class="text-sm text-slate-500">Loading…</p>
    <p
      v-else-if="codes.length === 0"
      class="text-sm text-slate-500 dark:text-slate-400"
    >
      You have no active recovery codes. Re-enroll TOTP to regenerate them.
    </p>
    <div
      v-else
      class="space-y-2 rounded-md border border-slate-200 bg-slate-50 p-4 font-mono text-sm dark:border-slate-800 dark:bg-slate-900"
    >
      <div v-for="c in codes" :key="c" class="select-all">{{ c }}</div>
    </div>
  </section>
</template>
