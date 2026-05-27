import "server-only";

import { cookies } from "next/headers";

import type { AllauthSession, AuthenticatedUser } from "~/lib/auth/types";

/**
 * Server-side session reader for App Router pages and Server Actions.
 *
 * Reads the user's `sessionid` cookie via `next/headers`, forwards it to
 * Django's allauth headless endpoint, and returns the canonical session
 * envelope. Marked `server-only` so accidentally importing into a Client
 * Component produces a build-time error (security gate — this file knows
 * the session cookie, the browser bundle must never see it).
 *
 * Pages call `requireUser()` at the top of `export default async function`.
 */

const BACKEND = process.env.NEXT_PUBLIC_BACKEND_URL ?? "http://localhost:8000";

export async function fetchSessionServer(): Promise<AllauthSession | null> {
  const cookieStore = await cookies();
  // Forward the entire cookie header (sessionid + csrftoken) so Django
  // can identify the user. We never log it.
  const cookieHeader = cookieStore
    .getAll()
    .map((c) => `${c.name}=${c.value}`)
    .join("; ");

  if (!cookieHeader) return null;

  try {
    const r = await fetch(`${BACKEND}/_allauth/browser/v1/auth/session`, {
      headers: {
        Accept: "application/json",
        Cookie: cookieHeader,
      },
      // Never cache auth state across requests / users.
      cache: "no-store",
    });
    const body = (await r.json()) as AllauthSession;
    body.status = r.status;
    return body;
  } catch {
    return null;
  }
}

/** Convenience: returns the user, or null if unauthenticated. */
export async function currentUser(): Promise<AuthenticatedUser | null> {
  const session = await fetchSessionServer();
  return session?.data?.user ?? null;
}
