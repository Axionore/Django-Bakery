<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";

import DefaultLayout from "~/layouts/DefaultLayout.vue";
import AccountLayout from "~/layouts/AccountLayout.vue";

const LAYOUTS = {
  default: DefaultLayout,
  account: AccountLayout,
} as const;

const route = useRoute();
const layout = computed(() => {
  const name = (route.meta.layout as keyof typeof LAYOUTS) || "default";
  return LAYOUTS[name] ?? DefaultLayout;
});
</script>

<template>
  <component :is="layout">
    <RouterView v-slot="{ Component }">
      <Suspense>
        <component :is="Component" />
      </Suspense>
    </RouterView>
  </component>
</template>
