import { Box, Callout, Flex, Heading, Text } from "@radix-ui/themes";
import { EnvelopeClosedIcon } from "@radix-ui/react-icons";

export function VerifyEmailPage() {
  return (
    <Flex direction="column" gap="4">
      <Heading size="6">Check your email</Heading>
      <Callout.Root>
        <Callout.Icon>
          <EnvelopeClosedIcon />
        </Callout.Icon>
        <Callout.Text>
          We&apos;ve sent you a confirmation link. Click it to verify your address — then come back
          and sign in.
        </Callout.Text>
      </Callout.Root>
      <Box>
        <Text size="2" color="gray">
          Tip: in local dev the email lands in <a href="http://localhost:8025" target="_blank" rel="noreferrer">Mailpit</a>.
        </Text>
      </Box>
    </Flex>
  );
}
