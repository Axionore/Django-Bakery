import Link from "next/link";
import { Button, Container, Flex, Heading, Text } from "@radix-ui/themes";

export const metadata = { title: "Not found" };

export default function NotFound() {
  return (
    <Container size="3">
      <Flex direction="column" gap="4" align="start" py="9">
        <Heading size="9">404</Heading>
        <Text size="4" color="gray">
          That page doesn&rsquo;t exist (or doesn&rsquo;t exist any more).
        </Text>
        <Button asChild>
          <Link href="/">Take me home</Link>
        </Button>
      </Flex>
    </Container>
  );
}
