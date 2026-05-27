<script setup lang="ts">
// Hydrate the auth store from /_allauth/browser/v1/auth/session on first mount.
// `useState`-backed Pinia store survives SSR -> client hydration.
const auth = useAuthStore();
const { $fetch: _$f } = useNuxtApp();

if (!auth.hydrated) {
  await auth.refresh();
}
</script>

<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>
