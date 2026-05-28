# 🥧 django-bakery

> A fast, modern, production-grade Django project generator written in Rust.

`django-bakery` is a single-binary CLI that bakes a complete, production-grade Django 6 project — **end-to-end**, **wired-up**, **OWASP A01–A10 + ASVS L2 baselined** — in roughly 150 ms. Every prompt option you'd hand-wire after `django-admin startproject` ships pre-configured: Django Ninja or DRF or GraphQL (Strawberry / Graphene); HTMX or React + Radix Themes or Vue 3 or Nuxt 4 or Next.js 16 (each in Full + Skeleton variants); PostgreSQL 18 / MySQL 9 / MariaDB LTS / SQLite; allauth + MFA TOTP + Argon2id + pwned-passwords + ratelimit; Celery + Redis; Sentry + OpenTelemetry + django-structlog; Docker Compose + Traefik + Let's Encrypt; GitHub Actions CI; type-checked via mypy or pyright. Live version resolution against PyPI + npm at bake time means the generated project pins actually-current versions, not whatever the template author thought was "latest" months ago.

```bash
$ django-bakery new
🥧  django-bakery v0.1.0  ·  Rust-powered Django scaffolding

? Project name              ▎ Acme
? Project slug              ▎ acme
? Python version            ▎ Python 3.14
? Django version            ▎ Django 6.0
? Primary mode              ▎ Production
? Primary relational DB     ▎ PostgreSQL 18
? API layer                 ▎ Django Ninja
? Frontend                  ▎ HTMX + Alpine.js
? CSS framework             ▎ Tailwind v4
? Celery                    ▎ yes  (Redis broker)
? Container setup           ▎ Docker Compose + Traefik + Let's Encrypt
? Version control           ▎ git
…

✔  Created ./acme  (187 files, 23 directories, 142 ms)

  Next steps:
    cd acme
    uv sync && uv run pre-commit install
    docker compose -f compose.local.yml up --build
```

---

## What it generates

Every generated project includes — **completely wired up**, no follow-up setup:

| Layer         | Default                                                             | Alternates                                                                    |
| ------------- | ------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| Backend       | Django 6 + Python 3.14                                              | Python 3.13                                                                   |
| Database      | PostgreSQL 18                                                       | SQLite / MySQL 8 / MariaDB 11                                                 |
| Graph DB      | _none_                                                              | Neo4j / NebulaGraph / SurrealDB / Dgraph                                      |
| API           | Django Ninja                                                        | DRF · GraphQL (Strawberry) · GraphQL (Graphene) · none                        |
| Frontend      | HTMX + Alpine.js                                                    | React + Vite + Radix Themes/Primitives · Nuxt 4 · Django templates · headless |
| JS language   | TypeScript 6+                                                       | JavaScript                                                                    |
| CSS           | Tailwind v4 (CSS-first `@theme`)                                    | Bootstrap 5 · none                                                            |
| Auth          | django-allauth + MFA TOTP + Argon2id                                | + headless mode for SPA frontends                                             |
| Tasks         | Celery + Beat + Flower (Redis)                                      | RabbitMQ broker                                                               |
| Email (dev)   | Mailpit                                                             | —                                                                             |
| Email (prod)  | Anymail + Mailgun                                                   | SES · SendGrid · SMTP · console                                               |
| Storage       | AWS S3                                                              | GCS · Azure Blob · WhiteNoise · nginx                                         |
| Observability | structlog + OpenTelemetry                                           | optional Sentry                                                               |
| Containers    | Docker Compose + Traefik + Let's Encrypt                            | Compose only · none                                                           |
| CI/CD         | GitHub Actions                                                      | GitLab CI · both · none                                                       |
| VCS           | git (`--initial-branch=main`)                                       | jj (git-colocated) · Mercurial · none                                         |
| Pre-commit    | ruff, ruff-format, djlint, gitleaks, codespell{% raw %}{% endraw %} | optional mypy/pyright                                                         |
| Type checks   | mypy + django-stubs                                                 | pyright · none                                                                |
| Toolchain     | uv + just                                                           | (no toggle)                                                                   |

### What's already secured (OWASP / ASVS L2 baseline)

- HSTS 1y + preload, secure cookies, `SECURE_REFERRER_POLICY=same-origin`
- CSP via `django-csp`, `X-Frame-Options=DENY`
- Argon2id password hashing (PBKDF2 retained for legacy migration)
- MFA (TOTP + recovery codes) ready to enable per-user
- Email verification mandatory for new signups
- Rate limiting on auth endpoints (`5 attempts / 5 min` default)
- ORM-only DB access (parameterized queries via `$1` / typed query builders)
- `django-cors-headers` deny-by-default; allowlist via env
- `pip-audit` + Trivy + `gitleaks` wired into CI
- Per-environment `.env.dev`, `.env.prod`, `.env.test` with explicit comments

## Latest-stable enforcement

At generation time, `django-bakery` queries PyPI and the npm registry for the latest stable version of every pinned dep, then writes them into the generated `pyproject.toml` / `package.json`. A compatibility check warns about known-bad pairs (e.g., django-stubs 5.x with Django 6.x). Pass `--offline` to skip and use bundled defaults.

```bash
django-bakery new                      # online — fetches latest
django-bakery new --offline            # offline — bundled defaults
django-bakery new --strict-compat      # fail on any compat warning
```

## Install

> **Current release status:** `v0.1.0-alpha.2`. Pre-built binaries + the multi-arch GHCR image are live; **crates.io + Homebrew are gated until the `v0.1.0` GA tag** (alpha tags intentionally skip those publish jobs so we don't pollute the public registries while iterating).

```bash
# Pre-built binary (Linux x86_64 / aarch64 musl, macOS x86_64 / aarch64, Windows x86_64)
curl -fsSL https://raw.githubusercontent.com/Axionore/Django-Bakery/main/installer/install.sh | sh

# Or grab a specific release tarball directly:
gh release download v0.1.0-alpha.2 --repo Axionore/Django-Bakery \
    --pattern '*aarch64-apple-darwin.tar.gz'
tar -xzf django-bakery-*.tar.gz
install -m 0755 django-bakery-* /usr/local/bin/django-bakery

# Docker (multi-arch — pulls linux/amd64 or linux/arm64 automatically)
docker run --rm -it -v "$PWD:/out" \
    ghcr.io/axionore/django-bakery:latest new --output /out
```

After GA lands (no ETA — gated on alpha sign-off):

```bash
cargo install django-bakery                                  # crates.io
brew install axionore/tap/django-bakery                      # Homebrew
```

Or build from source today:

```bash
git clone https://github.com/Axionore/Django-Bakery
cd Django-Bakery
cargo install --path crates/cli                              # → ~/.cargo/bin/django-bakery
```

Verify the install:

```bash
django-bakery --version
django-bakery --help
```

## Usage

### 1. Interactive — the 30-second path

```bash
django-bakery new
```

You'll be walked through prompts (project name → slug → stack → add-ons → containers). Every prompt has a sensible production default; press Enter through them all to get the recipe in [`What it generates`](#what-it-generates). The generator writes the project to `./<slug>/` and tells you the next three commands to run.

Pass `-o /path/to/parent` to control where it lands; `--yes` to accept every default without prompting (CI-friendly).

### 2. Recipe-driven — the reproducible path

For repeatable scaffolds (CI templates, internal starter kits, scripted onboarding), drive the generator from a TOML or JSON recipe file:

```bash
# Write a sample recipe with every option set to its default
django-bakery sample > recipe.toml

# Edit it
$EDITOR recipe.toml

# Validate it before rendering (catches enum typos, slug-shape errors,
# multi_tenant=true with non-Postgres, etc.)
django-bakery validate recipe.toml

# Render
django-bakery bake --config recipe.toml --output ./out

# Or in CI — re-render the same recipe deterministically
django-bakery bake --config recipe.toml --output ./out --offline --no-vcs --force
```

Want to start a prompted flow but pre-fill SOME answers? Pass `--preset`:

```bash
django-bakery new --preset team-defaults.toml
```

Anything in `team-defaults.toml` becomes the new default for that prompt; everything else still asks.

### 3. After generation — wiring the local dev loop

The next steps are printed by the generator, but here they are in full:

```bash
cd <your-project-slug>

# Backend deps (uv — fast, single source of truth in pyproject.toml + uv.lock)
uv sync

# Custom AUTH_USER_MODEL needs its initial migration on first boot
uv run python manage.py makemigrations users
uv run python manage.py migrate

# (Multi-tenant projects: use `migrate_schemas --shared` and bootstrap
#  the public Tenant — see docs/multi-tenancy.md inside the project)

uv run python manage.py createsuperuser

# Pre-commit hooks (ruff, ruff-format, djlint, gitleaks, codespell)
uv run pre-commit install
```

Run the stack however your `container_setup` choice produced:

```bash
# If you chose docker compose (the default):
docker compose -f compose.local.yml up --build

# If you chose `none`:
uv run python manage.py runserver
# (And for SPA frontends: `cd frontend && pnpm install && pnpm dev` in another tab.)
```

Hit `http://localhost:8000/`. The auto-generated project ships with:

- `/healthz/` — readiness probe (200 = ready)
- `/api/docs/` (Ninja or DRF) **or** `/api/graphql/` (Strawberry or Graphene) — interactive API docs
- `/admin/<random-suffix>/` — Django admin behind an unguessable URL (defends against blanket `/admin/` scanners)
- `/accounts/login/` — django-allauth flow, with MFA enrollment on first login for staff users (`STAFF_MFA_REQUIRED=True` by default)

The generated project's own `README.md`, `docs/deployment.md`, and (if multi-tenant) `docs/multi-tenancy.md` cover the rest of the lifecycle (deploys, Sentry/OTel wiring, staging vs prod env conventions).

### 4. Worked examples

**HTMX + Tailwind v4, server-rendered, SQLite — the minimal app:**

```bash
django-bakery new --yes -o ./scratch
# Defaults to: Ninja API, HTMX + Alpine.js, Tailwind v4, Postgres, compose+traefik.
# Override prompts to taste; or build a recipe with the overrides:
django-bakery sample > min.toml
$EDITOR min.toml  # set: frontend=htmx-alpine, relational_db=sqlite, container_setup=none
django-bakery bake --config min.toml --output ./scratch
```

**Multi-tenant SaaS (django-tenants, PG-schema-per-tenant):**

```bash
django-bakery sample > tenant.toml
# In tenant.toml:  relational_db = "postgres"
#                  multi_tenant   = true
#                  api_layer      = "drf"          # or "ninja"
#                  container_setup= "compose-traefik"
django-bakery bake --config tenant.toml --output ./acme-tenants

cd acme-tenants/acme_tenants
docker compose -f compose.local.yml up --build
# First boot runs: makemigrations users tenants → migrate_schemas --shared
# → bootstrap_public_tenant — see docs/multi-tenancy.md for the create_tenant
# command and the operator runbook.
```

**Full-stack Nuxt 4 SSR + Django Ninja:**

```bash
django-bakery sample > nuxt.toml
# In nuxt.toml: frontend = "nuxt", frontend_variant = "full", api_layer = "ninja"
django-bakery bake --config nuxt.toml --output ./acme

cd acme/acme
docker compose -f compose.local.yml up --build
# Django on :8000, Nuxt dev server on :3000, both sharing the .env, both with HMR.
# The Nuxt app already has the auth client wired against allauth-headless —
# session cookies, CSRF header, MFA branch, verify-email branch.
```

### CLI reference

```bash
django-bakery new                    # interactive (default subcommand)
django-bakery new --yes              # accept all defaults
django-bakery new --preset FILE      # pre-fill prompts from a recipe
django-bakery bake --config FILE     # non-interactive render from a recipe
django-bakery sample [--format toml|json]   # write a sample recipe
django-bakery validate FILE          # validate without rendering
django-bakery options                # JSON-schema-ish dump of every recipe option
```

Flags worth knowing (on `new` + `bake`):

| Flag               | Effect                                                                    |
| ------------------ | ------------------------------------------------------------------------- |
| `-o, --output DIR` | Parent directory for the generated project (default: `.`)                 |
| `--yes`            | Skip all prompts; use defaults (CI-friendly)                              |
| `--preset FILE`    | Pre-fill prompts from a saved recipe (`new` only)                         |
| `--offline`        | Skip PyPI/npm version checks; use bundled defaults snapshot               |
| `--strict-compat`  | Hard-fail on any compatibility warning (e.g. django-stubs major mismatch) |
| `--bootstrap`      | Run `uv sync` (+ `pnpm install`, `pre-commit install`) after generation   |
| `--force`          | Overwrite an existing non-empty output directory                          |
| `--no-vcs`         | Skip VCS init                                                             |
| `-v, -vv, -vvv`    | Increase log verbosity                                                    |

Verbose modes are useful when the resolver fails (`-vv` prints which PyPI/npm calls timed out + what default it fell back to).

## Architecture

A Cargo workspace with three crates:

```
django-bakery/
├── crates/
│   ├── cli/         # the `django-bakery` binary (clap + inquire)
│   ├── engine/      # render pipeline (minijinja + include_dir)
│   └── templates/   # the embedded Django template tree
└── installer/       # install.sh, Dockerfile
```

- **Templates** live as a real directory tree under `crates/templates/files/`, embedded into the binary via `include_dir!`.
- **Conditional inclusion** uses `__SKIP__` sentinels emitted by Jinja-in-filename — e.g., `{% if not bakery.use_celery %}__SKIP__{% endif %}config/celery_app.py.j2`. Renders to `__SKIP__config/...` → skipped; or `config/...` → included. Works at the directory level too.
- **Engine** is `minijinja` (Jinja2-compatible) — same `{{ }}` / `{% %}` syntax any Django developer already knows. The template-context namespace is a single flat `{{ bakery.<field> }}` — covering both your recipe answers (`bakery.project_slug`, `bakery.api_layer`, `bakery.multi_tenant`) and engine-computed extras (live-resolved pins via `bakery.versions['<pkg>']`, random secrets like `bakery.django_secret_key`, derived booleans like `bakery.is_postgres`).
- **Speed** comes from: native binary, no Python startup, single-pass walker, `include_dir` virtual FS.

## Contributing

```bash
cargo test
cargo run -- new -o /tmp/scratch --yes --force --offline
cargo clippy -- -D warnings
cargo fmt
```

Templates live in `crates/templates/files/{{bakery.project_slug}}/`. Conditional files use the Jinja-prefix pattern above. Snapshot tests live in `tests/snapshot/`.

## License

Dual-licensed under MIT or Apache-2.0 at your option.
