"use client";

import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";

type Appearance = "light" | "dark";

interface ThemeCtx {
  appearance: Appearance;
  setAppearance: (a: Appearance) => void;
  toggle: () => void;
}

const Ctx = createContext<ThemeCtx | null>(null);
const STORAGE_KEY = "ui.appearance";

function initial(): Appearance {
  if (typeof window === "undefined") return "light";
  const saved = window.localStorage.getItem(STORAGE_KEY);
  if (saved === "light" || saved === "dark") return saved;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

export function ThemeAppearanceProvider({ children }: { children: ReactNode }) {
  const [appearance, setAppearanceState] = useState<Appearance>("light");

  useEffect(() => {
    setAppearanceState(initial());
  }, []);

  const setAppearance = useCallback((a: Appearance) => {
    setAppearanceState(a);
    if (typeof window !== "undefined") window.localStorage.setItem(STORAGE_KEY, a);
  }, []);

  const toggle = useCallback(
    () => setAppearance(appearance === "dark" ? "light" : "dark"),
    [appearance, setAppearance],
  );

  useEffect(() => {
    if (typeof document !== "undefined") {
      document.documentElement.dataset["theme"] = appearance;
    }
  }, [appearance]);

  const value = useMemo(
    () => ({ appearance, setAppearance, toggle }),
    [appearance, setAppearance, toggle],
  );
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useThemeAppearance(): ThemeCtx {
  const ctx = useContext(Ctx);
  if (!ctx) throw new Error("useThemeAppearance outside ThemeAppearanceProvider");
  return ctx;
}
