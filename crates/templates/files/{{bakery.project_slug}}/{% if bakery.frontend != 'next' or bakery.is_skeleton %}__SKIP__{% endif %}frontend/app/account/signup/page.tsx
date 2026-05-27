"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { Button, Callout, Flex, Heading, Text, TextField } from "@radix-ui/themes";
import { ExclamationTriangleIcon } from "@radix-ui/react-icons";
import { z } from "zod";

import { authClient } from "~/lib/auth/client";
import { useAuthStore } from "~/lib/auth/store";

const schema = z
  .object({
    email: z.email("Enter a valid email"),
    password: z.string().min(12, "Password must be at least 12 characters"),
    confirm: z.string(),
  })
  .refine((data) => data.password === data.confirm, {
    message: "Passwords don't match",
    path: ["confirm"],
  });

export default function SignupPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string[]>>({});
  const [busy, setBusy] = useState(false);
  const router = useRouter();
  const setUser = useAuthStore((s) => s.setUser);

  async function onSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setFieldErrors({});
    const parsed = schema.safeParse({ email, password, confirm });
    if (!parsed.success) {
      setError(parsed.error.issues[0]?.message ?? "Check your inputs");
      return;
    }
    setBusy(true);
    const result = await authClient.signup(parsed.data.email, parsed.data.password);
    setBusy(false);
    switch (result.kind) {
      case "verification_sent":
        router.push("/account/verify-email");
        return;
      case "logged_in":
        setUser(result.user);
        router.push("/account/profile");
        return;
      case "duplicate_email":
        setError("An account already exists for that email.");
        return;
      case "validation_error":
        setFieldErrors(result.fields);
        setError("Check the highlighted fields and try again.");
        return;
      default:
        setError("Something went wrong. Please try again.");
    }
  }

  return (
    <Flex direction="column" gap="4">
      <Heading size="6">Create your account</Heading>
      {error ? (
        <Callout.Root color="red" role="alert">
          <Callout.Icon>
            <ExclamationTriangleIcon />
          </Callout.Icon>
          <Callout.Text>{error}</Callout.Text>
        </Callout.Root>
      ) : null}
      <form onSubmit={onSubmit} aria-label="Sign up">
        <Flex direction="column" gap="3">
          <Field label="Email" error={fieldErrors["email"]}>
            <TextField.Root
              type="email"
              autoComplete="email"
              required
              value={email}
              onChange={(e) => setEmail(e.currentTarget.value)}
            />
          </Field>
          <Field label="Password" error={fieldErrors["password"]} hint="At least 12 characters">
            <TextField.Root
              type="password"
              autoComplete="new-password"
              required
              value={password}
              onChange={(e) => setPassword(e.currentTarget.value)}
            />
          </Field>
          <Field label="Confirm password" error={fieldErrors["confirm"]}>
            <TextField.Root
              type="password"
              autoComplete="new-password"
              required
              value={confirm}
              onChange={(e) => setConfirm(e.currentTarget.value)}
            />
          </Field>
          <Button type="submit" size="3" loading={busy} disabled={busy}>
            Create account
          </Button>
        </Flex>
      </form>
      <Text size="2" color="gray">
        Already have an account? <Link href="/account/login">Sign in</Link>
      </Text>
    </Flex>
  );
}

function Field({
  label,
  hint,
  error,
  children,
}: {
  label: string;
  hint?: string;
  error?: string[];
  children: React.ReactNode;
}) {
  return (
    <label>
      <Text as="div" size="2" weight="medium" mb="1">
        {label}
      </Text>
      {children}
      {hint ? (
        <Text as="div" size="1" color="gray" mt="1">
          {hint}
        </Text>
      ) : null}
      {error && error.length > 0 ? (
        <Text as="div" size="1" color="red" mt="1">
          {error.join(" · ")}
        </Text>
      ) : null}
    </label>
  );
}
