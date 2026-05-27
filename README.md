# 🥧 django-bakery

> A fast, modern, production-grade Django project generator written in Rust.

`django-bakery` is a single-binary CLI that bakes a complete Django 6 project — **end-to-end**, **wired-up**, **OWASP-baselined** — in roughly 150 ms. Think `cookiecutter-django` but with native speed, 2026-current defaults, and the stack permutations cookiecutter never had (Django Ninja, GraphQL, Nuxt 4, React + Radix, graph databases).

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

```bash
# crates.io
cargo install django-bakery

# Homebrew
brew install charlesasobel/tap/django-bakery

# Pre-built binary (Linux / macOS / Windows)
curl -fsSL https://raw.githubusercontent.com/Axionore/Django-Barkery/main/installer/install.sh | sh

# Docker
docker run --rm -it -v "$PWD:/out" ghcr.io/Axionore/Django-Barkery new --output /out
```

Or build from source:

```bash
git clone https://github.com/Axionore/Django-Barkery
cd django-bakery
cargo install --path crates/cli
```

## Usage

```bash
django-bakery new                              # interactive
django-bakery new --yes                        # all defaults
django-bakery new --preset my.toml             # pre-fill from a recipe
django-bakery bake --config my.toml --output ./out
django-bakery sample --format toml > my.toml   # write a sample recipe
django-bakery validate my.toml                 # check a recipe
django-bakery options                          # show the full recipe schema
```

Flags worth knowing:

| Flag              | Effect                                                                  |
| ----------------- | ----------------------------------------------------------------------- |
| `--yes`           | Skip all prompts; use defaults                                          |
| `--preset FILE`   | Pre-fill prompts from a saved recipe                                    |
| `--offline`       | Skip PyPI/npm version checks; use bundled snapshot                      |
| `--strict-compat` | Hard-fail on any compatibility warning                                  |
| `--bootstrap`     | Run `uv sync` (+ `pnpm install`, `pre-commit install`) after generation |
| `--force`         | Overwrite an existing non-empty output directory                        |
| `--no-vcs`        | Skip VCS init                                                           |

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
- **Conditional inclusion** uses `__SKIP__` sentinels emitted by Jinja-in-filename — e.g., `{% if not cookiecutter.use_celery %}__SKIP__{% endif %}config/celery_app.py.j2`. Renders to `__SKIP__config/...` → skipped; or `config/...` → included. Works at the directory level too.
- **Engine** is `minijinja` (Jinja2-compatible). Templates port 1:1 from cookiecutter-django.
- **Speed** comes from: native binary, no Python startup, single-pass walker, `include_dir` virtual FS.

## Contributing

```bash
cargo test
cargo run -- new -o /tmp/scratch --yes --force --offline
cargo clippy -- -D warnings
cargo fmt
```

Templates live in `crates/templates/files/{{cookiecutter.project_slug}}/`. Conditional files use the Jinja-prefix pattern above. Snapshot tests live in `tests/snapshot/`.

## License

Dual-licensed under MIT or Apache-2.0 at your option.
