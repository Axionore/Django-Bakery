<script setup lang="ts">
import { computed } from "vue";
import { storeToRefs } from "pinia";

import { useAuthStore } from "~/stores/auth";

const auth = useAuthStore();
const { user } = storeToRefs(auth);
const mfaOn = computed(() => user.value?.has_usable_mfa === true);
</script>

<template>
  <section v-if="user" class="space-y-5 py-6">
    <header>
      <h1 class="text-3xl font-semibold">Your profile</h1>
      <p class="mt-1 text-sm text-slate-600 dark:text-slate-400">Signed in as {{ '{{' }} user.email {{ '}}' }}</p>
    </header>
    <div class="divide-y divide-slate-200 rounded-md border border-slate-200 dark:divide-slate-800 dark:border-slate-800">
      <div class="flex items-center justify-between px-4 py-3">
        <span class="text-sm text-slate-600 dark:text-slate-400">Email</span>
        <span class="text-sm">{{ '{{' }} user.email {{ '}}' }}</span>
      </div>
      <div class="flex items-center justify-between px-4 py-3">
        <span class="text-sm text-slate-600 dark:text-slate-400">Full name</span>
        <span class="text-sm">{{ '{{' }} user.full_name || "—" {{ '}}' }}</span>
      </div>
      <div class="flex items-center justify-between px-4 py-3">
        <span class="text-sm text-slate-600 dark:text-slate-400">Multi-factor auth</span>
        <span class="flex items-center gap-3 text-sm">
          <span
            :class="[
              'rounded-full px-2 py-0.5 text-xs font-medium',
              mfaOn
                ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-100'
                : 'bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-100',
            ]"
          >
            {{ '{{' }} mfaOn ? "Enrolled" : "Not enrolled" {{ '}}' }}
          </span>
          <RouterLink
            v-if="!mfaOn"
            to="/account/mfa-activate"
            class="rounded-md bg-slate-900 px-3 py-1 text-xs text-white hover:bg-slate-700 dark:bg-slate-100 dark:text-slate-900"
          >
            Enable
          </RouterLink>
          <RouterLink v-else to="/account/recovery-codes" class="underline">Recovery codes</RouterLink>
        </span>
      </div>
      <div v-if="user.is_staff" class="flex items-center justify-between px-4 py-3">
        <span class="text-sm text-slate-600 dark:text-slate-400">Role</span>
        <span class="rounded-full bg-blue-100 px-2 py-0.5 text-xs font-medium text-blue-800 dark:bg-blue-900 dark:text-blue-100">Staff</span>
      </div>
    </div>
  </section>
</template>
