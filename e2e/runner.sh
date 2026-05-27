#!/usr/bin/env bash
# Render → install → boot → ready-check. Playwright/screenshots are driven from
# the host (the MCP tool or an external Playwright runner), not by this script.
#
# Usage:   ./e2e/runner.sh <scenario>
#          ./e2e/runner.sh htmx-full
#          ./e2e/runner.sh react-full
#
# Side-effects: leaves Django (and the SPA dev server, if any) running. To stop,
# call ./e2e/runner.sh --stop or kill the pids in e2e/scratch/<scenario>/.pids.

set -euo pipefail

cd "$(dirname "$0")/.."

ROOT="$PWD"
E2E="$ROOT/e2e"
BIN="$ROOT/target/release/django-bakery"

usage() {
    echo "usage: $0 <scenario>" >&2
    echo "       $0 --stop [scenario]   stop background servers for a scenario (or all)" >&2
    echo
    echo "scenarios available:" >&2
    ls "$E2E/recipes/" | sed 's/\.toml$//' | sed 's/^/  /' >&2
    exit 64
}

[[ $# -lt 1 ]] && usage

if [[ "${1:-}" == "--stop" ]]; then
    target="${2:-}"
    for pidfile in "$E2E/scratch/"*/.pids; do
        [[ -f "$pidfile" ]] || continue
        name=$(basename "$(dirname "$pidfile")")
        if [[ -n "$target" && "$target" != "$name" ]]; then
            continue
        fi
        echo "↻  stopping $name"
        while read -r pid; do
            kill "$pid" 2>/dev/null || true
        done < "$pidfile"
        rm -f "$pidfile"
    done
    exit 0
fi

SCENARIO="$1"
RECIPE="$E2E/recipes/$SCENARIO.toml"
SCRATCH="$E2E/scratch/$SCENARIO"
LOGS="$E2E/logs/$SCENARIO"
[[ -f "$RECIPE" ]] || { echo "✘  no recipe: $RECIPE" >&2; exit 1; }
[[ -x "$BIN" ]] || { echo "✘  build first: cargo build --release -p django-bakery" >&2; exit 1; }

mkdir -p "$LOGS"

echo "===================================================================="
echo "scenario: $SCENARIO"
echo "recipe:   $RECIPE"
echo "scratch:  $SCRATCH"
echo "logs:     $LOGS"
echo "===================================================================="

# Stop ALL prior runs — every scenario shares Django :8765, so co-existing
# scenarios would silently land on whichever backend bound first.
"$0" --stop 2>/dev/null || true
# Belt-and-braces: kill any straggler holding :8765 or the SPA dev ports.
# `lsof -ti` misses ss-only-visible sockets on some kernels — `fuser -k` is the
# fallback that's reliable across discovery layers.
for port in 8765 5173 3000; do
    fuser -k "${port}/tcp" 2>/dev/null || true
done
sleep 1

rm -rf "$SCRATCH"
mkdir -p "$SCRATCH"

# --- 1) render -------------------------------------------------------------
echo "↻  rendering…"
"$BIN" bake --config "$RECIPE" --output "$SCRATCH" --no-vcs --force --offline

PROJ_DIR=$(find "$SCRATCH" -maxdepth 1 -mindepth 1 -type d | head -1)
[[ -n "$PROJ_DIR" ]] || { echo "✘  no rendered project dir" >&2; exit 1; }
echo "    → $PROJ_DIR"

# --- 2) backend deps -------------------------------------------------------
echo "↻  uv sync…"
(cd "$PROJ_DIR" && uv sync --quiet) > "$LOGS/uv-sync.log" 2>&1

cat > "$PROJ_DIR/.env" <<EOF
DJANGO_SETTINGS_MODULE=config.settings.local
DJANGO_SECRET_KEY=$(python3 -c "import secrets; print(secrets.token_urlsafe(48))")
DJANGO_DEBUG=True
DJANGO_ALLOWED_HOSTS=localhost,127.0.0.1
DATABASE_URL=sqlite:///./db.sqlite3
USE_DEBUG_TOOLBAR=False
STAFF_MFA_REQUIRED=False
PWNED_PASSWORDS_ENABLED=False
EOF

# --- 3a) makemigrations users (custom AUTH_USER_MODEL needs an initial migration)
echo "↻  makemigrations users…"
(cd "$PROJ_DIR" && uv run python manage.py makemigrations users --noinput) > "$LOGS/makemigrations.log" 2>&1

# --- 3b) migrate -----------------------------------------------------------
echo "↻  migrate…"
(cd "$PROJ_DIR" && uv run python manage.py migrate --noinput) > "$LOGS/migrate.log" 2>&1

# --- 3c) css build (htmx + tailwind only — others compile via Vite/Nuxt) ---
# A top-level `package.json` only exists when the recipe selected htmx-alpine
# + tailwind. Compile static/css/app.css → app.compiled.css before serving.
if [[ -f "$PROJ_DIR/package.json" ]]; then
    echo "↻  pnpm install (tailwind CLI)…"
    cat > "$PROJ_DIR/.npmrc" <<'EOF'
verify-deps-before-run=false
EOF
    (cd "$PROJ_DIR" && pnpm install --ignore-scripts --reporter=silent) \
        > "$LOGS/pnpm-install-root.log" 2>&1
    [[ -d "$PROJ_DIR/node_modules" ]] || { echo "✘  root pnpm install left no node_modules — see $LOGS/pnpm-install-root.log" >&2; exit 1; }
    echo "↻  tailwindcss build…"
    (cd "$PROJ_DIR" && pnpm css:build) > "$LOGS/css-build.log" 2>&1
    [[ -f "$PROJ_DIR/static/css/app.compiled.css" ]] || { echo "✘  app.compiled.css missing — see $LOGS/css-build.log" >&2; exit 1; }
fi

# --- 4) backend runserver in the background -------------------------------
echo "↻  starting Django on :8765…"
mkdir -p "$SCRATCH"
> "$SCRATCH/.pids"
cd "$PROJ_DIR"
nohup uv run python manage.py runserver --noreload 127.0.0.1:8765 \
    > "$LOGS/runserver.log" 2>&1 &
DJANGO_PID=$!
echo "$DJANGO_PID" >> "$SCRATCH/.pids"
cd "$ROOT"

# --- 5) wait for Django to answer ------------------------------------------
echo -n "↻  waiting for Django"
for _ in $(seq 1 30); do
    if curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:8765/healthz/ 2>/dev/null | grep -q 200; then
        echo " ✓"
        break
    fi
    sleep 1
    echo -n "."
done

# --- 6) frontend (if any) --------------------------------------------------
if [[ -d "$PROJ_DIR/frontend" ]]; then
    # Drop an .npmrc that opts out of pnpm 11's `verify-deps-before-run` and
    # build-script gate so dev-server boot is friction-free for the e2e harness.
    # These settings live in the rendered project too via _dot_npmrc.j2.
    cat > "$PROJ_DIR/frontend/.npmrc" <<'EOF'
verify-deps-before-run=false
EOF
    echo "↻  pnpm install (this can take a minute)…"
    # `--ignore-scripts` skips dependency lifecycle scripts (postinstall, etc.)
    # which is what trips pnpm 11's `ERR_PNPM_IGNORED_BUILDS`. The project's
    # own scripts (`pnpm dev`, `pnpm build`) still run normally below.
    (cd "$PROJ_DIR/frontend" && pnpm install --ignore-scripts --reporter=silent) > "$LOGS/pnpm-install.log" 2>&1
    [[ -d "$PROJ_DIR/frontend/node_modules" ]] || { echo "✘  pnpm install left no node_modules — check $LOGS/pnpm-install.log" >&2; exit 1; }

    case "$SCENARIO" in
        nuxt-*|next-*) FRONTEND_PORT=3000 ;;
        *) FRONTEND_PORT=5173 ;;
    esac

    echo "↻  starting frontend on :$FRONTEND_PORT…"
    cd "$PROJ_DIR/frontend"
    # Forward the chosen Django port to the dev-server proxy. Templates default
    # to :8000, which clashes with anything else (e.g. SurrealDB) bound there.
    # `npm_config_verify_deps_before_run=false` disables pnpm 11's pre-run
    # `pnpm install` that would otherwise re-trigger the `ERR_PNPM_IGNORED_BUILDS`
    # exit-1 we tolerated above.
    npm_config_verify_deps_before_run=false \
    VITE_BACKEND_URL=http://localhost:8765 \
    NITRO_BACKEND_URL=http://localhost:8765 \
    NUXT_PUBLIC_BACKEND_URL=http://localhost:8765 \
    NEXT_PUBLIC_BACKEND_URL=http://localhost:8765 \
    NEXT_INTERNAL_BACKEND_URL=http://localhost:8765 \
    nohup pnpm dev > "$LOGS/frontend-dev.log" 2>&1 &
    FE_PID=$!
    echo "$FE_PID" >> "$SCRATCH/.pids"
    cd "$ROOT"

    echo -n "↻  waiting for frontend"
    for _ in $(seq 1 60); do
        if curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:$FRONTEND_PORT/" 2>/dev/null | grep -qE '2..|3..'; then
            echo " ✓"
            break
        fi
        sleep 2
        echo -n "."
    done
fi

echo
echo "===================================================================="
echo "✔  ready"
echo "    Django:   http://127.0.0.1:8765"
[[ -d "$PROJ_DIR/frontend" ]] && echo "    Frontend: http://127.0.0.1:$FRONTEND_PORT"
echo "    Logs:    $LOGS/"
echo "    Pids:    $(cat $SCRATCH/.pids | tr '\n' ' ')"
echo
echo "  Run \`./e2e/runner.sh --stop $SCENARIO\` to stop."
echo "===================================================================="
