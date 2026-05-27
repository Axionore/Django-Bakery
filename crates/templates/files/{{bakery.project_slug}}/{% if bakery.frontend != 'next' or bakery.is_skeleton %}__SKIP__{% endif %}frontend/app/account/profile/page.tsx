import { redirect } from "next/navigation";
import Link from "next/link";
import { Badge, Box, Button, Container, Flex, Heading, Text } from "@radix-ui/themes";
import { LockClosedIcon } from "@radix-ui/react-icons";

import { currentUser } from "~/lib/auth/server";

export const metadata = { title: "Profile" };

/**
 * Server Component auth gate.
 *
 * The redirect happens on the SERVER — there's no flash of empty content, no
 * client-side waterfall, and the unauthenticated user never sees a request go
 * out for protected data. The Django backend remains the final authority for
 * authorization on every fetch the page initiates.
 */
export default async function ProfilePage() {
  const user = await currentUser();
  if (!user) {
    redirect("/account/login?next=/account/profile");
  }

  const mfaOn = user.has_usable_mfa === true;

  return (
    <Container size="3">
      <Flex direction="column" gap="5" py="6">
        <Box>
          <Heading size="6">Your profile</Heading>
          <Text size="2" color="gray">
            Signed in as {user.email}
          </Text>
        </Box>
        <Flex direction="column" gap="3">
          <Row label="Email" value={user.email} />
          <Row label="Full name" value={user.full_name || "—"} />
          <Row
            label="Multi-factor auth"
            value={
              <Flex align="center" gap="2">
                <Badge color={mfaOn ? "green" : "amber"}>
                  {mfaOn ? "Enrolled" : "Not enrolled"}
                </Badge>
                {!mfaOn ? (
                  <Button asChild size="1" variant="soft">
                    <Link href="/account/mfa-activate">
                      <LockClosedIcon /> Enable
                    </Link>
                  </Button>
                ) : (
                  <Button asChild size="1" variant="ghost">
                    <Link href="/account/recovery-codes">Recovery codes</Link>
                  </Button>
                )}
              </Flex>
            }
          />
          {user.is_staff ? <Row label="Role" value={<Badge color="blue">Staff</Badge>} /> : null}
        </Flex>
      </Flex>
    </Container>
  );
}

function Row({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <Flex
      justify="between"
      align="center"
      py="2"
      style={{ borderBottom: "1px solid var(--gray-a4)" }}
    >
      <Text size="2" color="gray">
        {label}
      </Text>
      <Text size="2">{value}</Text>
    </Flex>
  );
}
