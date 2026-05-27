"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Button, Callout, Flex, Heading, Text, TextField } from "@radix-ui/themes";
import { ExclamationTriangleIcon } from "@radix-ui/react-icons";
import { z } from "zod";

import { authClient } from "~/lib/auth/client";
import { useAuthStore } from "~/lib/auth/store";

const schema = z.object({
  email: z.email("Enter a valid email"),
  password: z.string().min(1, "Password is required"),
});

export default function LoginPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const router = useRouter();
  const params = useSearchParams();
  const setUser = useAuthStore((s) => s.setUser);

  async function onSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    const parsed = schema.safeParse({ email, password });
    if (!parsed.success) {
      setError(parsed.error.issues[0]?.message ?? "Invalid input");
      return;
    }
    setBusy(true);
    const result = await authClient.login(parsed.data.email, parsed.data.password);
    setBusy(false);
    switch (result.kind) {
      case "ok":
        setUser(result.user);
        router.push(params.get("next") ?? "/account/profile");
        return;
      case "mfa_required":
        router.push("/account/mfa-challenge");
        return;
      case "email_verification_required":
        router.push("/account/verify-email");
        return;
      case "invalid_credentials":
        setError("Incorrect email or password.");
        return;
      case "rate_limited":
        setError("Too many attempts. Try again in about 5 minutes.");
        return;
      default:
        setError("Something went wrong. Please try again.");
    }
  }

  return (
    <Flex direction="column" gap="4">
      <Heading size="6">Sign in</Heading>
      <Text size="2" color="gray">
        Use your email and password.
      </Text>
      {error ? (
        <Callout.Root color="red" role="alert">
          <Callout.Icon>
            <ExclamationTriangleIcon />
          </Callout.Icon>
          <Callout.Text>{error}</Callout.Text>
        </Callout.Root>
      ) : null}
      <form onSubmit={onSubmit} aria-label="Sign in">
        <Flex direction="column" gap="3">
          <label>
            <Text as="div" size="2" weight="medium" mb="1">
              Email
            </Text>
            <TextField.Root
              type="email"
              autoComplete="email"
              required
              value={email}
              onChange={(e) => setEmail(e.currentTarget.value)}
            />
          </label>
          <label>
            <Text as="div" size="2" weight="medium" mb="1">
              Password
            </Text>
            <TextField.Root
              type="password"
              autoComplete="current-password"
              required
              value={password}
              onChange={(e) => setPassword(e.currentTarget.value)}
            />
          </label>
          <Button type="submit" size="3" loading={busy} disabled={busy}>
            Sign in
          </Button>
        </Flex>
      </form>
      <Text size="2" color="gray">
        New here? <Link href="/account/signup">Create an account</Link>
      </Text>
    </Flex>
  );
}
