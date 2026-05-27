import { onMounted, ref, watch } from "vue";

type Mode = "light" | "dark";
const STORAGE_KEY = "ui.appearance";

/**
 * Dark-mode toggle with prefers-color-scheme fallback + localStorage persistence.
 *
 * Important: `localStorage` here holds ONLY the user's theme preference — NOT
 * any session/token/auth data. Auth state lives in Django's session cookie.
 */
export function useColorMode() {
  const mode = ref<Mode>("light");

  function readInitial(): Mode {
    if (typeof window === "undefined") return "light";
    const saved = window.localStorage.getItem(STORAGE_KEY);
    if (saved === "light" || saved === "dark") return saved;
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }

  function apply(next: Mode) {
    if (typeof document !== "undefined") {
      document.documentElement.classList.toggle("dark", next === "dark");
      document.documentElement.dataset.theme = next;
    }
  }

  onMounted(() => {
    mode.value = readInitial();
    apply(mode.value);
  });

  watch(mode, (next) => {
    if (typeof window !== "undefined") window.localStorage.setItem(STORAGE_KEY, next);
    apply(next);
  });

  function toggle() {
    mode.value = mode.value === "dark" ? "light" : "dark";
  }

  return { mode, toggle };
}
