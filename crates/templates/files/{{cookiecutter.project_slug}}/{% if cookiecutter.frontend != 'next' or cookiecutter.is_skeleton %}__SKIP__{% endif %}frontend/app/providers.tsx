"use client";

import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { Theme } from "@radix-ui/themes";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { ThemeAppearanceProvider, useThemeAppearance } from "~/lib/ui/theme";
import { useAuthStore } from "~/lib/auth/store";
import type { AuthenticatedUser } from "~/lib/auth/types";

interface Props {
  children: ReactNode;
  /** Seeded by the Server Component layout via `fetchSessionServer()`. */
  initialUser: AuthenticatedUser | null;
}

function ThemeWrapper({ children }: { children: ReactNode }) {
  const { appearance } = useThemeAppearance();
  return (
    <Theme
      appearance={appearance}
      accentColor="indigo"
      grayColor="slate"
      radius="medium"
      scaling="100%"
    >
      {children}
    </Theme>
  );
}

export function Providers({ children, initialUser }: Props) {
  // QueryClient must be stable across renders — never store outside React state.
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: { staleTime: 30_000, retry: 1, refetchOnWindowFocus: false },
          mutations: { retry: 0 },
        },
      }),
  );

  const setUser = useAuthStore((s) => s.setUser);
  useEffect(() => {
    setUser(initialUser);
  }, [initialUser, setUser]);

  return (
    <ThemeAppearanceProvider>
      <ThemeWrapper>
        <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
      </ThemeWrapper>
    </ThemeAppearanceProvider>
  );
}
