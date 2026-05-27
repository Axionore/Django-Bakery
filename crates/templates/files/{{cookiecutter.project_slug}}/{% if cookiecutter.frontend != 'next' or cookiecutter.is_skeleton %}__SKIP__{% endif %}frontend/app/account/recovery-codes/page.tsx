"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Box, Callout, Code, Flex, Heading, Text } from "@radix-ui/themes";
import { InfoCircledIcon } from "@radix-ui/react-icons";

import { authClient } from "~/lib/auth/client";
import { useAuth } from "~/lib/auth/store";

export default function RecoveryCodesPage() {
  const { user, status } = useAuth();
  const router = useRouter();
  const [codes, setCodes] = useState<string[] | null>(null);

  useEffect(() => {
    if (status === "loaded" && !user) {
      router.replace("/account/login?next=/account/recovery-codes");
    }
  }, [status, user, router]);

  useEffect(() => {
    if (!user) return;
    void (async () => {
      const r = await authClient.recoveryCodes();
      setCodes(r?.codes ?? []);
    })();
  }, [user]);

  if (!user) return null;

  return (
    <Flex direction="column" gap="4">
      <Heading size="6">Recovery codes</Heading>
      <Callout.Root color="amber">
        <Callout.Icon>
          <InfoCircledIcon />
        </Callout.Icon>
        <Callout.Text>
          Store these somewhere safe (1Password, a paper wallet). Each code works{" "}
          <strong>once</strong>. You can use one to sign in if you lose access to your authenticator
          app.
        </Callout.Text>
      </Callout.Root>
      {codes === null ? (
        <Text size="2" color="gray">
          Loading…
        </Text>
      ) : codes.length === 0 ? (
        <Text size="2" color="gray">
          You have no active recovery codes. Re-enroll TOTP to regenerate them.
        </Text>
      ) : (
        <Box style={{ background: "var(--gray-2)", padding: "1rem", borderRadius: "var(--radius-3)" }}>
          <Flex direction="column" gap="2">
            {codes.map((c) => (
              <Code key={c} variant="ghost" size="3">
                {c}
              </Code>
            ))}
          </Flex>
        </Box>
      )}
    </Flex>
  );
}
