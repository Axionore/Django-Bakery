<script setup lang="ts">
definePageMeta({ layout: "account" });
useSeoMeta({ title: "Verify your email" });
</script>

<template>
  <div class="space-y-4">
    <h1 class="text-2xl font-semibold">Check your email</h1>
    <div class="rounded-md border border-blue-200 bg-blue-50 p-4 text-sm text-blue-900 dark:border-blue-900 dark:bg-blue-950 dark:text-blue-100">
      We've sent you a confirmation link. Click it to verify your address — then come back and sign in.
    </div>
    <p class="text-sm text-slate-500 dark:text-slate-400">
      Tip: in local dev the email lands in
      <a href="http://localhost:8025" target="_blank" rel="noreferrer" class="underline">Mailpit</a>.
    </p>
  </div>
</template>
