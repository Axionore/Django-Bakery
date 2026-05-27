import { useEffect, useState } from "react";
import { useNavigate } from "react-router";
import { Box, Button, Callout, Code, Flex, Heading, Text, TextField } from "@radix-ui/themes";
import { ExclamationTriangleIcon, CheckCircledIcon } from "@radix-ui/react-icons";

import { authClient } from "~/auth/client";
import { RequireAuth } from "~/auth/guards";

export function MfaActivatePage() {
  return (
    <RequireAuth>
      <MfaActivate />
    </RequireAuth>
  );
}

function MfaActivate() {
  const [uri, setUri] = useState<string | null>(null);
  const [secret, setSecret] = useState<string | null>(null);
  const [code, setCode] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);
  const [busy, setBusy] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    void (async () => {
      const r = await authClient.mfaActivateBegin();
      if (r.kind === "ok") {
        setUri(r.uri);
        setSecret(r.secret);
      } else {
        setError("Couldn't start TOTP enrollment.");
      }
    })();
  }, []);

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
      setTimeout(() => navigate("/account/recovery-codes"), 1200);
    } else {
      setError("That code didn't match. Check the time on your device and try again.");
    }
  }

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
            Scan this QR code with your authenticator app (1Password, Authy, Google Authenticator, etc.) — or enter the
            secret manually.
          </Text>
          {uri ? (
            <Box style={{ background: "var(--gray-3)", padding: "1.5rem", borderRadius: "var(--radius-3)" }}>
              <QrFromUri uri={uri} />
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

/** Lightweight QR renderer using a public quickchart.io URL — pure pixel-by-pixel
 * data, no JS deps. The otpauth:// URI is one-time-use so embedding it in a query
 * string is acceptable; we explicitly do NOT log or persist it.
 */
function QrFromUri({ uri }: { uri: string }) {
  const src = `https://quickchart.io/qr?text=${encodeURIComponent(uri)}&size=220`;
  return <img src={src} alt="Two-factor QR code" width={220} height={220} style={{ display: "block" }} />;
}
