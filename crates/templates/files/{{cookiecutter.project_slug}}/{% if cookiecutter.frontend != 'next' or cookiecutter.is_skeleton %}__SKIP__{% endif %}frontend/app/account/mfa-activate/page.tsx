"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Box, Button, Callout, Code, Flex, Heading, Text, TextField } from "@radix-ui/themes";
import { ExclamationTriangleIcon, CheckCircledIcon } from "@radix-ui/react-icons";

import { authClient } from "~/lib/auth/client";
import { useAuth } from "~/lib/auth/store";

export default function MfaActivatePage() {
  const { user, status } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (status === "loaded" && !user) {
      router.replace("/account/login?next=/account/mfa-activate");
    }
  }, [status, user, router]);

  const [uri, setUri] = useState<string | null>(null);
  const [secret, setSecret] = useState<string | null>(null);
  const [code, setCode] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!user) return;
    void (async () => {
      const r = await authClient.mfaActivateBegin();
      if (r.kind === "ok") {
        setUri(r.uri);
        setSecret(r.secret);
      } else {
        setError("Couldn't start TOTP enrollment.");
      }
    })();
  }, [user]);

  async function onConfirm(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    if (code.length < 6) {
      setError("Enter the 6-digit code from your authenticator.");
      return;
    }
    setBusy(true);
    const r = await authClient.mfaActivateConfirm(code);
    setBusy(false);
    if (r.kind === "ok") {
      setDone(true);
      setTimeout(() => router.push("/account/recovery-codes"), 1200);
    } else {
      setError("That code didn't match. Check the time on your device and try again.");
    }
  }

  if (!user) return null;

  return (
    <Flex direction="column" gap="4">
      <Heading size="6">Set up two-factor auth</Heading>
      {done ? (
        <Callout.Root color="green">
          <Callout.Icon>
            <CheckCircledIcon />
          </Callout.Icon>
          <Callout.Text>MFA enabled. Redirecting to your recovery codes…</Callout.Text>
        </Callout.Root>
      ) : (
        <>
          <Text size="2" color="gray">
            Scan this QR code with an authenticator app (1Password, Authy, Google Authenticator) —
            or enter the secret manually.
          </Text>
          {uri ? (
            <Box
              style={{
                background: "var(--gray-3)",
                padding: "1.5rem",
                borderRadius: "var(--radius-3)",
              }}
            >
              <img
                src={`https://quickchart.io/qr?text=${encodeURIComponent(uri)}&size=220`}
                alt="Two-factor QR code"
                width={220}
                height={220}
                style={{ display: "block" }}
              />
              <Text as="div" size="1" color="gray" mt="3">
                Secret: <Code>{secret}</Code>
              </Text>
            </Box>
          ) : (
            <Text size="2" color="gray">
              Generating…
            </Text>
          )}
          {error ? (
            <Callout.Root color="red" role="alert">
              <Callout.Icon>
                <ExclamationTriangleIcon />
              </Callout.Icon>
              <Callout.Text>{error}</Callout.Text>
            </Callout.Root>
          ) : null}
          <form onSubmit={onConfirm} aria-label="Confirm MFA">
            <Flex direction="column" gap="3">
              <Text as="div" size="2" weight="medium">
                Enter the 6-digit code to confirm
              </Text>
              <TextField.Root
                inputMode="numeric"
                pattern="[0-9]*"
                autoComplete="one-time-code"
                maxLength={6}
                placeholder="123 456"
                value={code}
                onChange={(e) => setCode(e.currentTarget.value.replace(/[^0-9]/g, ""))}
              />
              <Button type="submit" size="3" loading={busy} disabled={busy}>
                Activate MFA
              </Button>
            </Flex>
          </form>
        </>
      )}
    </Flex>
  );
}
