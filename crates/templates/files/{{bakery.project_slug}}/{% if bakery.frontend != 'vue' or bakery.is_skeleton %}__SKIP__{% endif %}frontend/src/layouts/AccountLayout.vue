<template>
  <div class="flex min-h-dvh flex-col bg-slate-50 text-slate-900 dark:bg-slate-950 dark:text-slate-50">
    <main class="mx-auto w-full max-w-md flex-1 px-4 py-10">
      <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-800 dark:bg-slate-900">
        <slot />
      </div>
      <p class="mt-4 text-center text-xs text-slate-500">
        <RouterLink to="/" class="underline">← Back to {{ new Date().getFullYear() ? "home" : "" }}</RouterLink>
      </p>
    </main>
  </div>
</template>
