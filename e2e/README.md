# django-bakery end-to-end tests

Live tests that **actually render and boot** each recipe, then drive a browser via Playwright to take screenshots. This is the proof that "the generator produces a working Django project" — not just that the engine emits the right file shape (`crates/engine/tests/render.rs` already covers that).

## Layout

```
e2e/
├── recipes/         one TOML per scenario (htmx-full, react-full, …)
├── scratch/         where the generator drops the rendered projects (gitignored)
├── logs/            stdout/stderr per service per run (gitignored)
├── screenshots/     Playwright snapshots, committed for reference
├── runner.sh        render → install → boot → screenshot → cleanup
└── README.md
```

## Run a single scenario

```bash
./e2e/runner.sh htmx-full
./e2e/runner.sh react-full
./e2e/runner.sh nuxt-full
./e2e/runner.sh vue-full
./e2e/runner.sh next-full
./e2e/runner.sh react-skeleton    # skeletons too
```

Each run:

1. Renders the recipe to `e2e/scratch/<name>/`.
2. Runs `uv sync` for the Django backend (SQLite — no Docker needed).
3. Runs `python manage.py migrate` + `python manage.py runserver 127.0.0.1:8000` in the background.
4. If the recipe has a frontend, runs `pnpm install` + `pnpm dev` in the background.
5. Drives Playwright against the dev server: home, /about, /account/login, /account/signup, and the API docs URL.
6. Drops PNGs in `e2e/screenshots/<name>/`.
7. Stops the dev servers (`pkill -P $$` on the runner's PID).

## What we're checking visually

For every recipe:

- The Home page renders (no React/Vue/Next error overlay).
- The nav shows the project name + Sign in / Sign up links.
- `/account/login` shows the form with the project's branding.
- `/account/signup` likewise.
- The Permissions-Policy / X-Frame-Options headers reach the browser (only seen via network panel — the screenshot just shows the page).

The engine snapshot tests in `crates/engine/tests/render.rs` already prove the file shape is correct; these tests prove the _resulting code_ compiles + runs without error.

## CI

These tests run in `.github/workflows/ci.yml` against `ubuntu-latest`. They are gated behind a `e2e` label on PRs because they take ~5 min per scenario.

## What's deliberately NOT covered

- Full Docker Compose stack (Postgres + Mailpit + Celery + Redis + Traefik) — that's the deploy-readiness test, not the per-recipe smoke.
- Production builds (`pnpm build`) — covered by per-stack `pnpm build` invocation in `pnpm test:e2e` inside the generated project.
- Real MFA + email confirmation flow — needs a live SMTP catcher; covered separately in the docker-compose smoke run.
