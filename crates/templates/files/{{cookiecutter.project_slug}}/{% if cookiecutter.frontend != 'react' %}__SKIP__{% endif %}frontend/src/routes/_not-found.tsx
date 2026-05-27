import { Link } from "react-router";
import { Box, Button, Flex, Heading, Text } from "@radix-ui/themes";

export function NotFoundPage() {
  return (
    <Flex direction="column" gap="4" align="start" py="9">
      <Heading size="9">404</Heading>
      <Text size="4" color="gray">
        That page doesn&rsquo;t exist (or doesn&rsquo;t exist any more).
      </Text>
      <Box>
        <Button asChild>
          <Link to="/">Take me home</Link>
        </Button>
      </Box>
    </Flex>
  );
}
