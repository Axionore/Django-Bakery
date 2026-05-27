/**
 * Types for the django-allauth headless API shape (`/_allauth/browser/v1/...`).
 *
 * Hand-written from the public allauth headless docs — we intentionally avoid
 * the project's own OpenAPI generator here because allauth is upstream and its
 * schema doesn't ship in our backend's OpenAPI. Keep narrow; widen only as the
 * SPA actually needs more fields.
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
  /** Steps still required to reach an authenticated session. */
  pending?: boolean;
  /** MFA-specific: which authenticator types may be used. */
  types?: string[];
  is_pending?: boolean;
  provider?: { id: string; name: string };
}

export interface AllauthSession {
  meta: {
    is_authenticated: boolean;
  };
  data?: {
    user?: AuthenticatedUser;
    flows?: AllauthFlow[];
    methods?: Array<{ method: string; at: number; email?: string; reauthenticated?: boolean }>;
  };
  /** Status code as returned by the backend (handy for type-narrowing). */
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
  | {
      kind: "ok";
      /** otpauth:// URI for QR rendering — never log this; one-time use is fine. */
      uri: string;
      secret: string;
    }
  | { kind: "unknown_error"; status: number };
