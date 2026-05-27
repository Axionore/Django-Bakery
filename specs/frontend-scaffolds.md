# frontend-scaffolds: `django-bakery new` with frontend=react or frontend=nuxt produces a working SPA wired to the Django backend, with navigation, auth flows, and tests — no follow-up scaffolding required

**Stakeholder**: charlesasobel (Axionore)
**Status**: draft
**Owner**: @charlesasobel
**Target release**: django-bakery v0.2.0
**Tracking**: post-v0.1 follow-up

## Context

`django-bakery v0.1.0` shipped 7 commits covering the generator engine, the full Django backend template, OWASP audit fixes, tests (51 passing), and distribution scaffolding. The Frontend enum exposes `react`, `nuxt`, `htmx-alpine`, `django-templates`, and `none`. The first two are currently **prompt-only**: the recipe accepts the choice, the Django side installs `allauth.headless` and registers the `/_allauth/` JSON URL routes, but **no `frontend/` directory is emitted**. Picking React or Nuxt today gives a developer a backend ready for an SPA and nothing on the client.

This spec covers what the React and Nuxt 4 frontends MUST emit to count as "shipped" — full parity with what a developer would scaffold by hand via `pnpm create vite@latest`, `npx nuxi@latest init`, etc., PLUS the things that are normally manual: the Django integration (CSRF, headless auth wiring, dev proxy, Compose service).

## Problem

> "I picked React in the prompt and got no React code." — a real user, paraphrased from the previous turn of this conversation.

When a developer scaffolds a Django + SPA project from anything else today (cookiecutter-django, hand-rolled), they spend 4–8 hours plumbing the integration: setting up a Vite/Nuxt project in a sibling directory, configuring CORS + CSRF, wiring the SPA auth client against allauth-headless (or, more commonly, swapping allauth for JWT and reintroducing the JWT-in-localStorage anti-pattern), gluing Docker Compose so both halves run on `docker compose up`, and building the first auth + profile pages from scratch. Most never finish properly — they ship JWT-with-localStorage as the auth model because the headless allauth path is undocumented in the wild.

django-bakery v0.2.0 closes that loop: one prompt, one render, one working full-stack app that already knows how to log a user in, run their MFA challenge, and call a typed API client — without ever touching a JWT.

## Competitor baseline — the parity matrix

> Surveyed via the public docs of each scaffold's official starter, GitHub repo trees, and `pnpm create <starter>` / `nuxi init` baseline output as of 2026-05.

### Direct + adjacent competitors

| Scaffold | Backend integration | Auth strategy | What it ships |
| --- | --- | --- | --- |
| **cookiecutter-django** | Django templates only | server-rendered allauth | No SPA option |
| **`pnpm create vite@latest`** | none (frontend-only) | none | Bare TSX template, 1 component |
| **`npx nuxi@latest init`** | Nuxt + Nitro | none | Bare Nuxt 4, 1 page |
| **shadcn/ui starter** | Vite or Next | NextAuth (Next path) | Component library + theming, no backend wiring |
| **create-t3-app** | Next.js + tRPC | NextAuth (own backend) | Full Next + Prisma + tRPC + tailwind; not a Django integration |
| **TanStack Start** | Vinxi server | none | File-routed React with SSR; no auth scaffold |
| **Remix → React Router 7 starter** | Cloudflare/Node | none | Data-routing primitives only |
| **django-react-boilerplate** (GH) | Django + DRF | JWT in localStorage ❌ | CRA, jQuery, abandoned ~2022 |
| **Inertia.js (Django port)** | Django views | Django sessions | Server-driven SPA; no separate frontend tree |
| **Vue 3 official `create-vue`** | none | none | Bare Vue, optional router/Pinia/Vitest checkboxes |
| **Refine.dev** | any | any | Admin-panel CRUD generator; not general SPA |

### Parity matrix — features × scaffolds × us

Rows are every capability ANY competitor ships in their scaffold. Columns are us today (v0.1) and us at v0.2.

| Capability | cc-django | create-vite | nuxi init | create-t3 | shadcn-starter | **Us v0.1** | **Us v0.2** |
| --- | :-: | :-: | :-: | :-: | :-: | :-: | :-: |
| One-command project generation | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| TypeScript by default | ❌ | ✅ | ✅ | ✅ | ✅ | n/a | ✅ |
| Latest stable TS (6.x) | — | ✅ | ✅ | ✅ | ⚠️ | n/a | ✅ |
| File-based routing (Nuxt-style) | — | ❌ | ✅ | ✅ | ❌ | n/a | ✅ (Nuxt) / ⚠️ React Router v7 declarative |
| Layout + nav out of the box | ✅ | ❌ | ⚠️ stub | ✅ | ✅ | ✅ (HTMX path) | ✅ |
| Auth pages (login/signup/profile) | ✅ (server-rendered) | ❌ | ❌ | ✅ (NextAuth) | ❌ | ⚠️ server only | ✅ |
| MFA enrollment flow in UI | ❌ | ❌ | ❌ | ❌ | ❌ | ⚠️ server only | ✅ |
| Type-safe API client | ❌ | ❌ | ❌ | ✅ (tRPC) | ❌ | ❌ | ✅ (OpenAPI → ts) |
| Vitest pre-configured | ❌ | ⚠️ opt-in | ⚠️ opt-in | ✅ | ⚠️ | ❌ | ✅ |
| Playwright E2E pre-configured | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Tailwind v4 (CSS-first) | — | ❌ | ⚠️ via module | ⚠️ v3 | ✅ | ✅ (HTMX path) | ✅ |
| Radix Themes or Primitives | — | ❌ | ❌ | ❌ | ✅ Primitives | n/a | ✅ both |
| Dev-server hot reload through Docker | ⚠️ | ❌ | ❌ | ❌ | ❌ | ✅ (Django) | ✅ both |
| Single `docker compose up` runs everything | ⚠️ partial | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
| CSRF correctly handled for SPA | ❌ | n/a | n/a | n/a | n/a | n/a | ✅ |
| Session-based auth (no JWT in localStorage) | ✅ | n/a | n/a | varies | n/a | n/a | ✅ |
| ESLint + Prettier configured | ⚠️ | ✅ | ✅ | ✅ | ✅ | n/a | ✅ |
| pnpm workspace tying frontend+backend | ❌ | ❌ | ❌ | ❌ | ❌ | n/a | ✅ |
| OWASP Top 10 SPA defenses by default | ❌ | ❌ | ❌ | ⚠️ | ❌ | n/a | ✅ |
| Pre-wired CSP `connect-src` for SPA | ❌ | n/a | n/a | n/a | n/a | n/a | ✅ |
| Sentry frontend integration | ❌ | ❌ | ❌ | ❌ | ❌ | n/a | ✅ (opt-in) |
| Dark mode toggle wired | ❌ | ❌ | ❌ | ⚠️ | ✅ | n/a | ✅ |
| Generated README explains the SPA | ❌ | ⚠️ | ⚠️ | ✅ | ⚠️ | ⚠️ | ✅ |
| Storybook | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ (deliberately out — see Non-goals) |

### Per-competitor polish worth matching

| Competitor | Notable polish | Their gap (our opening) |
| --- | --- | --- |
| **create-t3-app** | Type-safe everything (tRPC contract); env-var validation via Zod; opinionated DX | Locked to Next + tRPC + Prisma; can't bring your own backend |
| **shadcn/ui starter** | Components copy-paste into the project (not a dep); excellent dark-mode + tokens | No backend; you bring your own auth |
| **Nuxt official starter** | File-routing, auto-imports, layouts; `@nuxt/devtools` | No auth; one bare page |
| **Inertia (Django)** | Single render path between Django views and Vue/React; no API contract at all | No client-side hydration story; no Vitest; tightly coupled |
| **cookiecutter-django** | Battle-tested Docker layout; Mailpit; deploy notes | Zero SPA story |

### Parity gaps we close in v0.2

- [ ] Emit a real `frontend/` directory for React+Vite+Radix and Nuxt 4 paths
- [ ] TypeScript 6+ by default, `tsconfig.json` strict, `noUncheckedIndexedAccess`
- [ ] React Router v7 (declarative routes) for the React path; Nuxt 4 file-routing for Nuxt
- [ ] Layout component with nav + auth-aware menu
- [ ] Pages: `/`, `/about`, `/account/login`, `/account/signup`, `/account/profile`, `/account/mfa`
- [ ] Auth client (typed) hitting `/_allauth/browser/v1/...`
- [ ] Auth store (zustand for React, Pinia for Nuxt) with route guards
- [ ] Vitest 8 + `@testing-library/react` (React) or `@nuxt/test-utils` (Nuxt) — at least 3 tests
- [ ] Playwright 1.50+ E2E — login + MFA enrollment happy path
- [ ] Dev proxy (`/api`, `/_allauth`) to Django on `:8000`
- [ ] `pnpm-workspace.yaml` at the repo root tying `frontend/` to the project
- [ ] Docker Compose service for the frontend dev server with HMR
- [ ] OpenAPI-generated typed API client (when api_layer=ninja or drf) via `openapi-typescript`
- [ ] Sentry browser SDK (opt-in, gated on the existing `use_sentry` recipe field)
- [ ] Dark-mode toggle persisted in localStorage with `prefers-color-scheme` fallback
- **Deliberately omitted:**
  - **Next.js / Remix / TanStack Start** — explicitly not in scope. React + Vite is the chosen SPA path; users wanting SSR for React should pick Nuxt 4 (which renders Vue and does SSR by default) or wait for a future spec. Rationale: one React-stack option keeps the parity floor narrow enough to actually hit; Nuxt covers the SSR case.
  - **Storybook** — out of scope for v0.2. The Radix Themes / Tailwind v4 token system already documents components; Storybook is one extra DX axis to maintain. Revisit in v0.3 if customers ask.
  - **Plain Vue (no Nuxt)** — same logic as Next.js. Nuxt 4 IS the Vue path. A separate "Vue + Vite" option duplicates the parity surface for marginal value.
  - **GraphQL client** (Relay / urql / Apollo) — only relevant when `api_layer=graphql-*`. v0.2 ships the REST/OpenAPI client; GraphQL clients land in a follow-up spec along with the GraphQL templates themselves.
  - **PWA / offline mode** — no competitor scaffold ships this either; not the differentiator we'd pick.

## Our differentiator — what makes ours #1

Built on top of full parity:

- **The headless-allauth + Django + SPA loop, pre-wired** — no other scaffold knows how to integrate Django's mature session+MFA auth with an SPA. Every alternative pushes you to JWT-in-localStorage (an OWASP A02 anti-pattern) or to a JS-only backend. We ship a typed client against `/_allauth/browser/v1/...` that handles email login, MFA challenge, recovery codes, password reset, and email-verification gating — out of the box, all session-cookie-based, all CSRF-correct.
- **Type-safe API client generated from the project's own OpenAPI spec** — when the recipe picks `api_layer=ninja` (or `drf` with `drf-spectacular`), we run `openapi-typescript` at first install to produce `frontend/src/api/schema.d.ts` and ship a typed `fetcher.ts` wrapper. New endpoint on the Django side → re-run one command → typed client updated. create-t3 ties you to tRPC; we let you keep REST and still get the types.
- **One-command full-stack dev experience** — `just up` brings up Django + Postgres + Redis + Mailpit + the frontend's Vite/Nuxt dev server, all with HMR, all on a single Docker Compose stack, all sharing the `.env`. cookiecutter-django can't do this; create-t3 doesn't have a Django to talk to; the official Vite/Nuxt starters don't know about your backend.
- **SPA defenses baked in by default** — strict CSP `connect-src` honoring the configured backend host; CSRF token forwarding wired into the auth client (no developer can accidentally turn it off); `credentials: "include"` everywhere; route guards for staff routes that mirror the backend's `RequireMfaForStaffMiddleware`; Permissions-Policy aligned between client and server.

## Non-goals

- **Server-side rendering for the React path.** React + Vite is a pure SPA; for SSR pick Nuxt (Vue) or wait for a future spec.
- **Replacing Django's auth** with anything client-managed (NextAuth, Clerk, Supabase). The whole point is to keep allauth + sessions canonical and treat the SPA as a client.
- **Backend-for-frontend (BFF) layer.** We talk to Django directly; no extra Node service.
- **Per-component design system docs site.** Use the generated `docs/` folder for prose; ship working examples in the app itself.
- **Mobile (React Native / Capacitor / NativeScript / Quasar).** Web SPA only.
- **i18n beyond a stub.** `LOCALE_PATHS` already exist on the backend; the frontend ships `useI18n` interface stubs but actual translation files are out of scope.

## Security requirements (Secure SDLC — Defining phase)

- **Data classification** — PII via the User model (email, full_name). No payment data. No special-category data. Stored exclusively on the backend; the SPA only ever holds it in memory or in `localStorage` for *non-sensitive* UI state (dark-mode toggle, last-visited tab).
- **Regulatory scope** — GDPR applies to the PII path. No PCI, no HIPAA, no SOX.
- **ASVS target** — OWASP **ASVS 5.0 L2**, in-scope chapters: V3 (Session), V4 (Access Control), V5 (Validation/Sanitization/Encoding), V6 (Stored Crypto — for the password validators we already added), V8 (Data Protection), V11 (Business Logic), V13 (API), V14 (Configuration). Every requirement at L2 in these chapters becomes a Testing-phase test case.
- **Encryption** — in transit: TLS 1.3 enforced by the backend (`SECURE_SSL_REDIRECT`, HSTS); at rest: handled by the existing Django settings — the SPA stores nothing sensitive at rest.
- **Access control** — session-cookie based (`SessionTokenStrategy` in allauth headless). The SPA NEVER stores a JWT or session token in `localStorage` / `sessionStorage`. Route guards on the client are advisory UX — the backend remains the authority for every authorization decision. Cross-tenant access is server-enforced (no client-side scoping is trusted).
- **Authentication** — email + password via `/_allauth/browser/v1/auth/login`. MFA (TOTP + recovery codes) via `/_allauth/browser/v1/auth/2fa/authenticate`. Email verification mandatory before the SPA reveals authenticated routes. Login attempts are rate-limited by `ACCOUNT_RATE_LIMITS["login_failed"] = "5/5m"` on the backend.
- **Code & data integrity** — `pnpm` lockfile committed; `pnpm audit --prod` runs in CI; SRI hashes on any CDN-loaded asset (we avoid CDN where possible — Tailwind v4 + Radix come in via npm). Source maps NOT shipped to production. Webpack/Vite chunk hashing enabled.
- **Input validation** — every client-side input is validated by the SPA (`zod` schemas) AND by the backend (the source of truth). Client-side validation is UX, never security.
- **Data minimization** — the SPA fetches only the fields it renders; `/_allauth/browser/v1/auth/session` returns the minimal session envelope.
- **Logging & monitoring** — frontend errors go to Sentry (when enabled); auth events are logged backend-side via the `apps/users/signals.py` we already shipped. No tokens, no passwords, no full PII in client logs. Sentry's `beforeSend` strips emails.
- **Threat model** — REQUIRED (touches auth). See `specs/frontend-scaffolds.threat-model.md`.

### SPA-specific OWASP mapping

| OWASP A0X | Concern | Defense in this scaffold |
| --- | --- | --- |
| **A01 — Broken Access Control** | Client-side route guard can be bypassed | Treat client guards as UX-only; backend enforces |
| **A02 — Cryptographic Failures** | Storing session/JWT in localStorage (XSS = total takeover) | Session cookies only; `HttpOnly` on `sessionid`; SPA never reads the cookie |
| **A03 — Injection** | XSS via uncontrolled HTML rendering | React + Vue auto-escape; raw-HTML APIs linted off |
| **A04 — Insecure Design** | Mixing auth concerns into the SPA | Single auth-client module; route guards declarative |
| **A05 — Misconfiguration** | CORS `*` or credentials with wildcard origin | Strict `CORS_ALLOWED_ORIGINS` (env-driven), `credentials: "include"` only on same-origin in prod (the SPA is served from the Django host or a known sibling domain) |
| **A06 — Vulnerable Components** | `pnpm` deps go stale | `pnpm audit` in CI; Renovate weekly; pinned majors |
| **A07 — Auth Failures** | Missing MFA challenge in SPA flow | The auth client handles 401 + `data.flows[].id === "mfa_authenticate"` → renders the MFA challenge screen |
| **A08 — Integrity Failures** | Untrusted Vite/Nuxt plugins | Pin every dep by major; no `latest`; `pnpm.overrides` whitelist |
| **A09 — Logging Failures** | Sensitive data shipped to Sentry | `beforeSend` strips email/PII; Sentry token in env, not in code |
| **A10 — SSRF** | n/a in the SPA itself | Backend already covered |

## Acceptance criteria

A reviewer runs this list against a freshly generated project and verifies each.

### Generation
- [ ] `django-bakery new --yes --offline --recipe '<react preset>'` emits a `frontend/` directory at the project root
- [ ] `frontend/package.json` has `react@^19`, `react-dom@^19`, `vite@^7`, `typescript@^6`, `@radix-ui/themes@^4` (or `@radix-ui/react-*` primitives for the alternative flavor), `react-router@^7`, `zustand@^5`, `@tanstack/react-query@^5`, `zod@^4`, `vitest@^8`, `@playwright/test@^1.50`, `eslint@^9`, `prettier@^3`
- [ ] `frontend/package.json` for Nuxt has `nuxt@^4.1`, `vue@^3.6`, `typescript@^6`, `@nuxtjs/tailwindcss@^7`, `pinia@^3`, `@pinia/nuxt`, `vitest@^8`, `@playwright/test`, `@nuxt/test-utils`
- [ ] `pnpm-workspace.yaml` exists at the project root naming `frontend`
- [ ] No file in `frontend/` contains the literal string `__SKIP__` (extends the existing `rendered_files_never_contain_skip_sentinel` test to the frontend tree)
- [ ] The dotfile-shadow convention extends to frontend dotfiles: `_dot_env.example` → `frontend/.env.example`

### First-run developer experience
- [ ] `pnpm install` in `frontend/` succeeds with zero errors and zero `vulnerabilities (high)` from `pnpm audit`
- [ ] `pnpm dev` boots the Vite/Nuxt dev server on `:5173` (React) or `:3000` (Nuxt)
- [ ] `pnpm build` produces a deployable `dist/` (React) or `.output/` (Nuxt) with no TypeScript errors
- [ ] `pnpm test` runs Vitest with at least 3 passing tests
- [ ] `pnpm playwright test` runs E2E and at least the login-success path passes
- [ ] `docker compose -f compose.local.yml up` starts Django, Postgres, Redis (if Celery), Mailpit (if enabled), AND the frontend dev server, all with HMR

### Functional — pages render
- [ ] `/` shows a Home page with the project name, description, a "Sign in" CTA (when unauthenticated) or "Profile" link (when authenticated)
- [ ] `/about` shows the project description and stack info
- [ ] `/account/login` accepts email + password, POSTs to `/_allauth/browser/v1/auth/login`, and on 401-with-MFA-required transitions to the MFA challenge
- [ ] `/account/signup` accepts email + password + confirm, POSTs to `/_allauth/browser/v1/auth/signup`, shows "verify your email" on success
- [ ] `/account/profile` (auth-required) shows the calling user's email, full_name, and an MFA enrollment status badge
- [ ] `/account/mfa` allows TOTP enrollment with a QR code rendered from the backend response, and shows recovery codes once
- [ ] A 404 page exists for unknown routes

### Functional — auth client
- [ ] The auth client never reads or writes any token to `localStorage` or `sessionStorage`
- [ ] Every fetch sets `credentials: "include"` and `X-CSRFToken` from the `csrftoken` cookie
- [ ] On 401 with a `flows[].id === "verify_email"` response, the client redirects to a "check your email" page rather than the generic login error
- [ ] On 401 with a `flows[].id === "mfa_authenticate"`, the client transitions to the MFA challenge component

### Differentiators delivered
- [ ] Typed API client at `frontend/src/api/schema.d.ts` regenerates via `pnpm openapi:gen` (when `api_layer=ninja` or `drf`) from `/api/openapi.json`
- [ ] Dark-mode toggle in the nav persists in localStorage and respects `prefers-color-scheme` on first visit
- [ ] Generated `frontend/README.md` documents: where the dev server runs, how to regen the API client, where the auth client lives, and the explicit "no JWT in localStorage" policy

### Security gates (must pass before Beta sign-off)
- [ ] `pnpm audit --prod --json | jq '.metadata.vulnerabilities.high + .critical'` returns `0`
- [ ] No raw-HTML React API (the danger* one) in any source file — ESLint rule wired
- [ ] No `v-html` in any Nuxt component — ESLint rule wired
- [ ] CSP `connect-src` in `production.py.j2` includes the SPA origin (when configured)
- [ ] Playwright E2E proves: anonymous user hitting `/account/profile` is redirected to `/account/login`
- [ ] Playwright E2E proves: after login, the `sessionid` cookie is present and HttpOnly via `expect(cookie.httpOnly).toBe(true)`

### Engine tests
- [ ] New integration tests `frontend_react_recipe_emits_full_tree` and `frontend_nuxt_recipe_emits_full_tree` covering all of the above as path-presence + content assertions
- [ ] Existing `default_recipe_renders_full_stack` test still green (the default recipe stays on HTMX-Alpine; nothing changes for it)

## Design

### File trees

**React + Vite + Radix Themes path** — emitted when `frontend = "react"` and (depending on `radix_flavor`) either Themes or Primitives + Tailwind. Tree below is the Themes flavor; Primitives flavor swaps `@radix-ui/themes` for `@radix-ui/react-*` packages and adds Tailwind v4.

```
{{bakery.project_slug}}/
├── pnpm-workspace.yaml           # NEW — packages: ["frontend"]
├── frontend/
│   ├── package.json
│   ├── tsconfig.json             # strict, noUncheckedIndexedAccess, exactOptionalPropertyTypes
│   ├── tsconfig.node.json
│   ├── vite.config.ts            # proxies /api + /_allauth → http://django:8000
│   ├── eslint.config.js          # @typescript-eslint, react-hooks, no-raw-html-api
│   ├── .prettierrc
│   ├── _dot_env.example          # VITE_API_URL, VITE_SENTRY_DSN (opt)
│   ├── _dot_gitignore
│   ├── index.html
│   ├── playwright.config.ts
│   ├── vitest.config.ts
│   ├── README.md
│   ├── src/
│   │   ├── main.tsx              # createRoot, Radix Theme provider, Router provider, QueryClient
│   │   ├── App.tsx
│   │   ├── router.tsx            # react-router v7 routes
│   │   ├── routes/
│   │   │   ├── _layout.tsx       # nav, auth-aware menu, dark-mode toggle, footer
│   │   │   ├── index.tsx         # Home
│   │   │   ├── about.tsx
│   │   │   ├── account/
│   │   │   │   ├── _layout.tsx   # centered card layout for auth screens
│   │   │   │   ├── login.tsx
│   │   │   │   ├── signup.tsx
│   │   │   │   ├── verify-email.tsx
│   │   │   │   ├── mfa-challenge.tsx
│   │   │   │   ├── mfa-activate.tsx
│   │   │   │   ├── profile.tsx
│   │   │   │   └── recovery-codes.tsx
│   │   │   └── _not-found.tsx
│   │   ├── auth/
│   │   │   ├── client.ts         # typed fetch wrapper for /_allauth/browser/v1/...
│   │   │   ├── csrf.ts           # reads csrftoken cookie + sets X-CSRFToken header
│   │   │   ├── store.ts          # zustand auth store; user, session, flows
│   │   │   ├── guards.tsx        # <RequireAuth>, <RequireMfaEnrolled>, <RedirectIfAuthed>
│   │   │   ├── types.ts          # generated from the allauth headless OpenAPI later
│   │   │   └── tests/
│   │   │       ├── client.test.ts
│   │   │       └── store.test.ts
│   │   ├── api/
│   │   │   ├── client.ts         # typed fetch using schema.d.ts when present
│   │   │   ├── schema.d.ts       # openapi-typescript output; checked in, regenerated via script
│   │   │   └── tests/client.test.ts
│   │   ├── ui/
│   │   │   ├── theme.tsx         # dark-mode context + persistence
│   │   │   ├── nav.tsx
│   │   │   ├── empty-state.tsx
│   │   │   └── form-field.tsx    # Radix-themed wrapper with zod validation surface
│   │   ├── env.ts                # zod-validated env vars
│   │   └── styles/globals.css    # Radix theme tokens + project overrides
│   └── tests/
│       └── e2e/
│           ├── login.spec.ts
│           ├── signup-then-mfa.spec.ts
│           └── auth-guard.spec.ts
```

**Nuxt 4 path:**

```
{{bakery.project_slug}}/
├── pnpm-workspace.yaml
├── frontend/
│   ├── package.json
│   ├── tsconfig.json
│   ├── nuxt.config.ts            # nitro proxy to Django; pinia, tailwind v4 module, sentry
│   ├── app.vue
│   ├── _dot_env.example
│   ├── _dot_gitignore
│   ├── eslint.config.js          # @nuxt/eslint + no-v-html
│   ├── .prettierrc
│   ├── README.md
│   ├── playwright.config.ts
│   ├── vitest.config.ts
│   ├── layouts/
│   │   ├── default.vue
│   │   └── account.vue
│   ├── pages/                    # file-based routing
│   │   ├── index.vue
│   │   ├── about.vue
│   │   └── account/
│   │       ├── login.vue
│   │       ├── signup.vue
│   │       ├── verify-email.vue
│   │       ├── mfa-challenge.vue
│   │       ├── mfa-activate.vue
│   │       ├── profile.vue
│   │       └── recovery-codes.vue
│   ├── composables/
│   │   ├── useAuth.ts            # wraps useFetch + Pinia auth store
│   │   ├── useCsrf.ts
│   │   └── useApi.ts             # typed against schema.d.ts
│   ├── middleware/
│   │   ├── auth.global.ts        # default-deny on /account/profile + descendants
│   │   └── guest.ts              # redirect signed-in users away from /account/login
│   ├── stores/
│   │   └── auth.ts               # pinia
│   ├── server/
│   │   └── api/                  # (intentionally empty — we proxy to Django)
│   ├── types/
│   │   ├── allauth.d.ts
│   │   └── api-schema.d.ts       # openapi-typescript output
│   ├── assets/css/main.css
│   └── tests/
│       ├── stores/auth.test.ts
│       ├── pages/login.test.ts
│       └── e2e/
│           ├── login.spec.ts
│           ├── signup-then-mfa.spec.ts
│           └── auth-guard.spec.ts
```

### Engine changes

Templates use the existing `{% if bakery.frontend == 'react' %}...{% else %}__SKIP__{% endif %}` and `{% if bakery.frontend == 'nuxt' %}...{% else %}__SKIP__{% endif %}` prefix patterns. Add to context:

- `bakery.frontend_dev_port` — `5173` for React, `3000` for Nuxt
- `bakery.frontend_origin` — `http://localhost:<port>`
- `bakery.has_typed_api` — `True` when `api_layer in {ninja, drf}` (drives the `openapi-typescript` wiring)

No new Rust filters needed; existing `slugify`/`snake_case` cover the frontend file generation.

### Backend integration changes

- `config/settings/base.py.j2` already adds `allauth.headless`. We additionally:
  - Extend `CSP_CONNECT_SRC` to include the SPA origin in dev (read from env)
  - Set `CSRF_TRUSTED_ORIGINS` default to include `http://localhost:<frontend_dev_port>`
  - Add `CORS_ALLOWED_ORIGINS` default to include the SPA origin (env-overridable)
- `compose.local.yml.j2` adds a `frontend` service (Node 24 alpine + pnpm) that runs `pnpm dev`, mounts `./frontend`, exposes the dev port, and `depends_on: { django: { condition: service_started } }`
- When `api_layer = ninja`, add a script in the frontend's `package.json`: `"openapi:gen": "openapi-typescript http://django:8000/api/openapi.json -o src/api/schema.d.ts"`

### Auth client — the critical path

```ts
// frontend/src/auth/client.ts — sketch
export class AuthClient {
  constructor(private base = "/_allauth/browser/v1") {}

  async login(email: string, password: string): Promise<LoginResult> {
    const r = await fetch(`${this.base}/auth/login`, {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "application/json", "X-CSRFToken": getCsrfToken() },
      body: JSON.stringify({ email, password }),
    });
    if (r.status === 401) {
      const body: AllAuthSessionResponse = await r.json();
      if (body.meta?.is_authenticated === false && body.data?.flows) {
        const mfa = body.data.flows.find((f) => f.id === "mfa_authenticate");
        if (mfa) return { kind: "mfa_required", flow: mfa };
        const verify = body.data.flows.find((f) => f.id === "verify_email");
        if (verify) return { kind: "email_verification_required" };
      }
      return { kind: "invalid_credentials" };
    }
    if (!r.ok) throw new HttpError(r.status, await r.text());
    return { kind: "ok", session: await r.json() };
  }
  // signup, mfaAuthenticate, mfaActivate, logout, session, … similarly
}
```

(The shipped version is fully typed against the headless OpenAPI schema, has retry-on-CSRF-mismatch, and exposes a state-machine-friendly Result type.)

### Error states

| Surface | Failure | UX | Logged |
| --- | --- | --- | --- |
| Login | Bad creds (401, no flows) | Inline form error: "Incorrect email or password." | server side (existing signals) |
| Login | Bad creds + rate-limited (429) | Banner: "Too many attempts. Try again in ~5 minutes." | server side |
| Login | Network error | Toast: "Couldn't reach the server. Retry?" + Retry button | Sentry |
| Login | MFA required | Auto-navigate to `/account/mfa-challenge`, preserve session draft | n/a |
| Signup | Email in use (409) | Inline form error: "An account exists for this email." + "Sign in instead" link | n/a |
| Profile | Not authenticated | `<RequireAuth>` redirects to `/account/login?next=/account/profile` | n/a |
| MFA activate | Wrong code | Inline error; do not log the submitted code | server side |
| Any | 5xx | Toast + Sentry breadcrumb (no PII) | Sentry |

### Out-of-band

- **Feature flag**: this whole feature is gated on the recipe's `frontend` field; no runtime flag needed.
- **API client generation**: the OpenAPI fetch script runs on `pnpm install` via a `prepare` lifecycle hook so first-run is one command.
- **Sentry**: opt-in. When `use_sentry = true` AND `frontend = react|nuxt`, the SPA's `package.json` includes `@sentry/react` or `@sentry/nuxt`; init in `main.tsx` / `nuxt.config.ts` reads `VITE_SENTRY_DSN` / `NUXT_PUBLIC_SENTRY_DSN`.
- **Bundle size budget**: gzipped main bundle ≤ 250 KB for React, ≤ 350 KB for Nuxt initial chunk. Enforced by a Vite/Nuxt build step that fails CI on overrun.

## Rollout (Secure SDLC — Deploying phase: Alpha → Beta → GA)

This spec ships as `django-bakery v0.2.0`. The audience here is **the developer running `django-bakery new`**, not end-users — but the same gate pattern applies because the scaffolds become production code for *their* end-users.

**Pre-stage** — implement on a feature branch `feat/frontend-scaffolds`; render both recipes locally; run `pnpm install && pnpm build && pnpm test` against the rendered project; run Playwright against `docker compose up`.

### Alpha — internal dogfood
- Audience: Charles + 2–3 design partners running `django-bakery new` against this branch
- Duration: 48h
- Watch: render time (< 200 ms still), `pnpm install` time (< 60s warm cache), engine test suite (51 + ~10 new), generated-project `pnpm build` exit code
- **Alpha gate sign-off**:
  - [ ] Both React + Nuxt recipes render → install → build → dev-serve → playwright login spec passes
  - [ ] Engine tests (61 expected) all green
  - [ ] No P1/P2 bugs open; security-reviewer + owasp-auditor parallel run on the diff is clean
  - [ ] Signed off by: ______________  Date: ________

### Beta — public preview
- Audience: tag `v0.2.0-beta.1`; announce in README under "experimental" tag
- Duration: 1 week soak; track GitHub Issues for either recipe
- Watch: Issues tagged `frontend-react` and `frontend-nuxt`; CI smoke render of both recipes nightly
- **Beta gate sign-off**:
  - [ ] No Critical/High bugs reported during the soak
  - [ ] `pnpm audit --prod --json` shows zero High/Critical for both recipes' lockfiles
  - [ ] Signed off by: ______________  Date: ________

### GA — `v0.2.0` release
- Audience: 100% — the new options become the default-recommendation in the README's stack table
- **GA gate sign-off**:
  - [ ] Beta gate passed; rollback plan executable
  - [ ] Both recipes documented in README; the existing "scoped out of v0.1" note removed
  - [ ] Signed off by: ______________  Date: ________

## Rollback

- **If a recipe is broken**: revert the React or Nuxt context branch only (`Frontend::React` / `Frontend::Nuxt`) → the prompt still accepts the choice but errors with a clear "this recipe is temporarily disabled in vX.Y.Z; see #N" message. HTMX-Alpine and Django-Templates paths are untouched.
- **If the integration is unsafe** (e.g. a discovered XSS in the auth client): yank the v0.2 binary from GH Releases + Homebrew tap; emit a v0.2.1 with the recipe disabled; advisory CVE if material.
- **If `pnpm install` breaks** (transitive vuln in a pinned dep): bump the pin via Dependabot; release a v0.2.x patch.

## Tests required (ship in same PR)

- [ ] **Success path** — both recipes render → install → build → dev → playwright login OK
- [ ] **Auth-failure path** — login with bad creds: 401 surfaces as inline error
- [ ] **MFA challenge path** — staff user enrolls TOTP; subsequent login goes through challenge
- [ ] **Validation-failure path** — signup with too-short password: zod schema rejects client-side AND backend rejects (422)
- [ ] **Auth-guard path** — anonymous GET `/account/profile` → redirect to login
- [ ] **CSRF-rotation path** — server rotates the CSRF token; SPA's next mutating request re-reads the cookie and retries once
- [ ] **localStorage hygiene path** — Playwright asserts `localStorage` contains no key matching `/session|token|jwt|auth/i` after a login flow
- [ ] **Bundle-size assertion** — Vite build step fails if gzipped main chunk > 250 KB (React) / 350 KB (Nuxt)
- [ ] **Engine snapshot** — at least 6 new integration tests in `crates/engine/tests/render.rs`:
  - `frontend_react_recipe_emits_full_tree`
  - `frontend_nuxt_recipe_emits_full_tree`
  - `frontend_recipes_carry_no_skip_markers`
  - `frontend_dotfile_shadow_extends_to_subtree`
  - `pnpm_workspace_yaml_present_for_spa_recipes`
  - `csp_connect_src_extended_for_spa_origins`

## Open questions

- **Routing library for React** — React Router v7 (declarative, the safer ecosystem call) vs. TanStack Router (better typed routes, smaller mindshare). **Proposal**: React Router v7 for v0.2; revisit if a customer asks for TanStack in v0.3.
- **State management for React** — zustand (chosen here for size + DX) vs. Jotai vs. Redux Toolkit Query. **Proposal**: zustand for auth/session only; TanStack Query for server state.
- **Where does the `openapi-typescript` step live** — `prepare` script (runs on `pnpm install`) vs. a manual `pnpm openapi:gen` command. **Proposal**: manual command + a CI step that fails if `schema.d.ts` is stale relative to `/api/openapi.json`. Predictable, no surprise network calls during `pnpm install`.
- **Sentry frontend SDK enablement** — auto-enable when `use_sentry=true`, or add a separate `use_sentry_frontend` toggle. **Proposal**: auto-enable. If the user said "yes to Sentry" they meant the whole stack.
- **Vue 3 without Nuxt** — re-confirm it's out. **Proposal**: yes, out. Nuxt 4 covers Vue.
- **Storybook** — re-confirm it's out. **Proposal**: yes, out for v0.2. v0.3 if asked.
- **Hot-module reload through Docker Compose volume mounts on macOS/Windows** — known flaky. **Proposal**: emit a `.dockerignore` that excludes `node_modules`; document the `VITE_HMR_HOST` workaround in the generated frontend README.
