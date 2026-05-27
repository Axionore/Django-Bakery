# Changelog

All notable changes to this project are documented in this file. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions
follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2026-05-27

First public release. A single 5 MB Rust binary that generates a complete,
production-grade Django 6 project — OWASP A01–A10 baselined, multi-tenant
ready, every prompt option boots end-to-end with zero manual cleanup.

### Stack the generator can produce

- **Backend** — Django 6 on Python 3.14 / 3.13, async ORM, ASGI via uvicorn,
  gunicorn for sync, custom email-as-username `User` model.
- **Databases** — PostgreSQL 18 / MySQL 9 / MariaDB LTS / SQLite, each proven
  to boot end-to-end through the e2e harness.
- **API layers** — Django Ninja (default, async) / DRF + drf-spectacular /
  Strawberry GraphQL / Graphene GraphQL, OpenAPI 3.1 docs wired for both
  Ninja and DRF, GraphiQL for both GraphQL flavors.
- **Frontends** — HTMX + Alpine + Tailwind v4 / React + Vite + Radix Themes /
  Nuxt 4 SSR / Vue 3 + Vite + Pinia / Next.js 16 App Router. Each in a Full
  variant (auth + UI + tests) or a Skeleton (bare wireframe). All 9
  permutations boot end-to-end.
- **Auth** — django-allauth (session + headless modes for SPA frontends) +
  MFA TOTP + Argon2id password hashing + pwned-passwords k-anonymity
  validator + django-ratelimit on auth endpoints + MFA-enforced staff
  middleware.
- **Tasks** — Celery + django-celery-beat + django-celery-results + Flower,
  Redis or RabbitMQ broker, proven against a containerized Redis 8.
- **Observability** — Sentry SDK + OpenTelemetry SDK + django-structlog +
  OTLP exporter, all composable via the `use_observability` toggle.
- **Email** — Anymail (Mailgun, SES, SendGrid, Mailjet, Mandrill, Postmark,
  Brevo, SparkPost) + SMTP + console.
- **Storage** — AWS S3 / GCS / Azure Blob / WhiteNoise / nginx-served.
- **Container** — Docker Compose with Traefik + Let's Encrypt, or
  compose-only, or no containers. Multi-stage non-root Dockerfile with
  HEALTHCHECK, frozen uv lockfile, pinned uv version.
- **CI** — GitHub Actions / GitLab CI / both / none. Workflow YAML is
  structurally tested by the engine suite.
- **Multi-tenancy** — opt-in `django-tenants` (PG-schema-per-tenant) with a
  pre-scaffolded `apps/tenants/` app, `Tenant` + `Domain` models, and a
  full operator runbook in `docs/multi-tenancy.md`.
- **Feature flags** — django-waffle option.
- **Pre-commit** — ruff, ruff-format, djlint, codespell, gitleaks.

### Differentiators

- **Live version resolution** at bake-time against PyPI + npm. Templates
  read `^{{ bakery.versions.<pkg> }}` for ~77 packages across
  `pyproject.toml`, all 4 SPA Full `package.json` files, all 4 SPA Skeleton
  `package.json` files, and the HTMX root. A 77-package phantom-version
  regression test prevents bad pins from sneaking back into the bundled
  defaults snapshot.
- **5-channel distribution** — `cargo install`, Homebrew tap, GHCR Docker
  image, cross-platform GH Release binaries (Linux x86/ARM musl, macOS
  x86/ARM, Windows x64), and a `curl … | sh` installer.
- **e2e harness shipped in the repo** — `e2e/runner.sh` boots Django +
  the SPA dev server per scenario; `e2e/compose-runner.sh` does the full
  `docker compose up --build` + healthcheck cycle. 17 scenarios proven
  bootable, 50+ committed screenshots.
- **OWASP A01–A10:2025 baseline** wired and engine-tested — CSP / HSTS /
  strict CORS / HttpOnly cookies / Argon2id / MFA TOTP / pwned-passwords /
  rate limiting / Postgres bound to 127.0.0.1 / container HEALTHCHECK /
  multi-stage non-root Dockerfile. ASVS L2 target.
- **Generation speed** — ~150 ms (Rust binary + `include_dir!` + minijinja).
- **Tree-shape snapshots** (`insta`) for 4 canonical recipes catch any
  accidental add/remove across the whole rendered tree.

### Engine

- 100 tests (29 unit + 67 integration + 4 insta snapshots).
- minijinja 2.x for Jinja2-compatible rendering.
- `include_dir!` macro embeds the entire template tree at compile time.
- `__SKIP__` sentinel pattern in Jinja-rendered filenames for conditional
  inclusion at file + directory level.
- `_dot_X` → `.X` dotfile shadow convention (host git can't read template
  `.gitignore` / `.dockerignore` etc.).
- Online resolver (`bakery.versions.*`) with offline fallback to a
  truth-tested bundled snapshot.

### Known scoping calls (deliberately deferred)

- `username_type = username | email` toggle. The modern best practice is
  email-as-username (which is what we ship by default); supporting both
  would touch the User model + manager + allauth + factories + tests
  across 7 files for marginal user value.
- Plugin system / external recipe registry. v0.1.0 ships a single curated
  recipe set.
- Windows-native integration tests. Docker-heavy recipes are skipped on
  Windows in the release pipeline.
- Multi-tenant scaffolding beyond schema isolation (per-tenant feature flags,
  per-tenant async task queues). Foundations are present; specialized layers
  are project-specific.

[0.1.0]: https://github.com/Axionore/Django-Barkery/releases/tag/v0.1.0
