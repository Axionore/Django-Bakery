import { useState } from "react";
import { useNavigate } from "react-router";
import { Button, Callout, Flex, Heading, Text, TextField } from "@radix-ui/themes";
import { ExclamationTriangleIcon } from "@radix-ui/react-icons";

import { authClient } from "~/auth/client";
import { useAuthStore } from "~/auth/store";

export function MfaChallengePage() {
  const [code, setCode] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const navigate = useNavigate();
  const setUser = useAuthStore((s) => s.setUser);

  async function onSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    if (code.length < 6) {
      setError("Enter the 6-digit code from your authenticator app.");
      return;
    }
    setBusy(true);
    const result = await authClient.mfaAuthenticate(code);
    setBusy(false);
    switch (result.kind) {
      case "ok":
        setUser(result.user);
        navigate("/account/profile");
        return;
      case "invalid_credentials":
        setError("That code didn't match. Try again.");
        return;
      case "rate_limited":
        setError("Too many attempts. Wait a few minutes.");
        return;
      default:
        setError("Couldn't verify. Try again.");
    }
  }

  return (
    <Flex direction="column" gap="4">
      <Heading size="6">Two-factor code</Heading>
      <Text size="2" color="gray">
        Enter the 6-digit code from your authenticator app.
      </Text>
      {error ? (
        <Callout.Root color="red" role="alert">
          <Callout.Icon>
            <ExclamationTriangleIcon />
          </Callout.Icon>
          <Callout.Text>{error}</Callout.Text>
        </Callout.Root>
      ) : null}
      <form onSubmit={onSubmit} aria-label="MFA code">
        <Flex direction="column" gap="3">
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
            Verify
          </Button>
        </Flex>
      </form>
    </Flex>
  );
}
