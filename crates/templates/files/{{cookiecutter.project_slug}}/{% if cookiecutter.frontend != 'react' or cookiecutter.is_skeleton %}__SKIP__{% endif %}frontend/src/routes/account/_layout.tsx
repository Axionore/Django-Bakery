import { Outlet } from "react-router";
import { Box, Container, Flex } from "@radix-ui/themes";

export function AccountLayout() {
  return (
    <Container size="1" py="6">
      <Flex direction="column" gap="5">
        <Box asChild p="6" style={{ background: "var(--color-panel)", borderRadius: "var(--radius-4)" }}>
          <section>
            <Outlet />
          </section>
        </Box>
      </Flex>
    </Container>
  );
}
