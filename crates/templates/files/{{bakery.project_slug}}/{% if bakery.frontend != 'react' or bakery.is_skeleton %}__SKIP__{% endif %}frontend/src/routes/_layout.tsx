import { Outlet } from "react-router";
import { Box, Container, Flex } from "@radix-ui/themes";

import { Nav } from "~/ui/nav";
import { useSessionBootstrap } from "~/auth/store";

export function RootLayout() {
  // Hydrate the auth store from /_allauth/browser/v1/auth/session on first mount.
  // This is non-blocking: pages render immediately, the store flips when the
  // response lands. Guarded pages wait on `session.status === "loaded"`.
  useSessionBootstrap();

  return (
    <Flex direction="column" minHeight="100dvh">
      <Nav />
      <Box asChild flexGrow="1">
        <main>
          <Container size="3" px={{ initial: "4", sm: "6" }} py="6">
            <Outlet />
          </Container>
        </main>
      </Box>
      <Box asChild>
        <footer>
          <Container size="3" px={{ initial: "4", sm: "6" }} py="4">
            <Flex justify="between" align="center" style={{ color: "var(--gray-10)", fontSize: 12 }}>
              <span>© {new Date().getFullYear()} · powered by django-bakery</span>
              <a href="https://github.com/Axionore/Django-Barkery" target="_blank" rel="noreferrer">
                source
              </a>
            </Flex>
          </Container>
        </footer>
      </Box>
    </Flex>
  );
}
