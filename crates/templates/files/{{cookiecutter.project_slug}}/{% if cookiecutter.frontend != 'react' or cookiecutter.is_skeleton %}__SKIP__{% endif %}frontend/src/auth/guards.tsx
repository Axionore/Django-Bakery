import type { ReactNode } from "react";
import { Navigate, useLocation } from "react-router";
import { Spinner } from "@radix-ui/themes";

import { useAuth } from "~/auth/store";

interface GuardProps {
  children: ReactNode;
}

/**
 * Client-side route guard for the SPA.
 *
 * IMPORTANT: this is UX-only. The Django backend remains the authority for
 * authorization — every API/page request is gated by Django session+role checks.
 * A determined user could disable this guard in their devtools, but they'd hit
 * a 403/302 from the server immediately.
 */
export function RequireAuth({ children }: GuardProps) {
  const { user, status } = useAuth();
  const location = useLocation();
  if (status === "idle" || status === "loading") {
    return (
      <div style={{ padding: "4rem", textAlign: "center" }}>
        <Spinner size="3" />
      </div>
    );
  }
  if (!user) {
    return <Navigate to={`/account/login?next=${encodeURIComponent(location.pathname)}`} replace />;
  }
  return <>{children}</>;
}

/** Redirect already-signed-in users away from login/signup. */
export function RedirectIfAuthed({ children }: GuardProps) {
  const { user, status } = useAuth();
  if (status === "idle" || status === "loading") {
    return (
      <div style={{ padding: "4rem", textAlign: "center" }}>
        <Spinner size="3" />
      </div>
    );
  }
  if (user) return <Navigate to="/account/profile" replace />;
  return <>{children}</>;
}
