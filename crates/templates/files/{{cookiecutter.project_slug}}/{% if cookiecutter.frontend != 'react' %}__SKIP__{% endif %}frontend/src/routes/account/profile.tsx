import { Link } from "react-router";
import { Badge, Box, Button, Flex, Heading, Text } from "@radix-ui/themes";
import { LockClosedIcon } from "@radix-ui/react-icons";

import { useAuth } from "~/auth/store";
import { RequireAuth } from "~/auth/guards";

export function ProfilePage() {
  return (
    <RequireAuth>
      <ProfileContent />
    </RequireAuth>
  );
}

function ProfileContent() {
  const { user } = useAuth();
  if (!user) return null;
  const mfaOn = user.has_usable_mfa === true;
  return (
    <Flex direction="column" gap="5">
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
              <Badge color={mfaOn ? "green" : "amber"}>{mfaOn ? "Enrolled" : "Not enrolled"}</Badge>
              {!mfaOn ? (
                <Button asChild size="1" variant="soft">
                  <Link to="/account/mfa-activate">
                    <LockClosedIcon /> Enable
                  </Link>
                </Button>
              ) : (
                <Button asChild size="1" variant="ghost">
                  <Link to="/account/recovery-codes">View recovery codes</Link>
                </Button>
              )}
            </Flex>
          }
        />
        {user.is_staff ? <Row label="Role" value={<Badge color="blue">Staff</Badge>} /> : null}
      </Flex>
    </Flex>
  );
}

function Row({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <Flex justify="between" align="center" py="2" style={{ borderBottom: "1px solid var(--gray-a4)" }}>
      <Text size="2" color="gray">
        {label}
      </Text>
      <Text size="2">{value}</Text>
    </Flex>
  );
}
