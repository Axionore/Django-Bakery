import type { ReactNode } from "react";
import { Box, Container, Flex } from "@radix-ui/themes";

export default function AccountLayout({ children }: { children: ReactNode }) {
  return (
    <Container size="1" py="6">
      <Flex direction="column" gap="5">
        <Box p="6" style={{ background: "var(--color-panel)", borderRadius: "var(--radius-4)" }}>
          {children}
        </Box>
      </Flex>
    </Container>
  );
}
