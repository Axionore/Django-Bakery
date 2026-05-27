/**
 * Narrow types for the django-allauth headless API (`/_allauth/browser/v1/...`).
 *
 * Hand-written from the public allauth headless docs — allauth doesn't ship in
 * the project's own OpenAPI spec. Keep narrow; widen only when the SPA actually
 * needs more fields.
 */

export interface AuthenticatedUser {
  id: number;
  email: string;
  full_name: string;
  is_staff?: boolean;
  has_usable_mfa?: boolean;
}

export type AllauthFlowId =
  | "verify_email"
  | "login"
  | "signup"
  | "provider_redirect"
  | "provider_signup"
  | "provider_token"
  | "mfa_authenticate"
  | "mfa_reauthenticate"
  | "mfa_trust"
  | "reauthenticate"
  | "password_reset"
  | "password_reset_by_code";

export interface AllauthFlow {
  id: AllauthFlowId;
  pending?: boolean;
  is_pending?: boolean;
  types?: string[];
  provider?: { id: string; name: string };
}

export interface AllauthSession {
  meta: { is_authenticated: boolean };
  data?: {
    user?: AuthenticatedUser;
    flows?: AllauthFlow[];
    methods?: Array<{ method: string; at: number; email?: string; reauthenticated?: boolean }>;
  };
  status?: number;
}

export type LoginResult =
  | { kind: "ok"; user: AuthenticatedUser }
  | { kind: "mfa_required"; flow: AllauthFlow }
  | { kind: "email_verification_required" }
  | { kind: "invalid_credentials" }
  | { kind: "rate_limited" }
  | { kind: "unknown_error"; status: number };

export type SignupResult =
  | { kind: "verification_sent" }
  | { kind: "logged_in"; user: AuthenticatedUser }
  | { kind: "duplicate_email" }
  | { kind: "validation_error"; fields: Record<string, string[]> }
  | { kind: "unknown_error"; status: number };

export type MfaActivateBeginResult =
  | { kind: "ok"; uri: string; secret: string }
  | { kind: "unknown_error"; status: number };
