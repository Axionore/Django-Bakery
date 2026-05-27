#!/usr/bin/env bash
# Compose-lifecycle e2e: render a recipe with `container_setup != none`, then
# `docker compose -f compose.local.yml up --build -d`, wait for healthchecks,
# probe Django, and tear down. Proves the rendered Docker Compose stack
# actually boots end-to-end — separate concern from the per-process e2e runner
# (`runner.sh`) which boots Django directly with `manage.py runserver`.
#
# Usage:   ./e2e/compose-runner.sh <scenario>
#          ./e2e/compose-runner.sh compose-full
#          ./e2e/compose-runner.sh --stop [scenario]

set -euo pipefail

cd "$(dirname "$0")/.."

ROOT="$PWD"
E2E="$ROOT/e2e"
BIN="$ROOT/target/release/django-bakery"

usage() {
    echo "usage: $0 <scenario>" >&2
    echo "       $0 --stop [scenario]   tear down compose stacks" >&2
    echo
    echo "scenarios available:" >&2
    ls "$E2E/recipes/" | sed 's/\.toml$//' | sed 's/^/  /' >&2
    exit 64
}

[[ $# -lt 1 ]] && usage

if [[ "${1:-}" == "--stop" ]]; then
    target="${2:-}"
    for project_dir in "$E2E/compose-scratch/"*/; do
        [[ -d "$project_dir" ]] || continue
        name=$(basename "$project_dir")
        if [[ -n "$target" && "$target" != "$name" ]]; then continue; fi
        proj_root=$(find "$project_dir" -maxdepth 1 -mindepth 1 -type d | head -1)
        [[ -n "$proj_root" ]] || continue
        if [[ -f "$proj_root/compose.local.yml" ]]; then
            echo "↻  tearing down $name"
            local_files=(-f compose.local.yml)
            # `compose.e2e.yml` is our explicit harness-only override (it is
            # NOT named `compose.override.yml` — that filename is auto-loaded
            # by docker compose and would leak ephemeral port remaps into
            # whatever any user later runs from this rendered project tree).
            [[ -f "$proj_root/compose.e2e.yml" ]] && local_files+=(-f compose.e2e.yml)
            (cd "$proj_root" && docker compose "${local_files[@]}" down -v --remove-orphans 2>&1) | tail -5 || true
        fi
    done
    exit 0
fi

SCENARIO="$1"
RECIPE="$E2E/recipes/$SCENARIO.toml"
SCRATCH="$E2E/compose-scratch/$SCENARIO"
LOGS="$E2E/compose-logs/$SCENARIO"

[[ -f "$RECIPE" ]] || { echo "✘  no recipe: $RECIPE" >&2; exit 1; }
[[ -x "$BIN" ]] || { echo "✘  build first: cargo build --release -p django-bakery" >&2; exit 1; }
command -v docker >/dev/null || { echo "✘  docker not on PATH" >&2; exit 1; }
docker compose version >/dev/null 2>&1 || { echo "✘  'docker compose' v2 plugin not installed" >&2; exit 1; }

mkdir -p "$LOGS"

echo "===================================================================="
echo "compose-scenario: $SCENARIO"
echo "recipe:           $RECIPE"
echo "scratch:          $SCRATCH"
echo "logs:             $LOGS"
echo "===================================================================="

# Tear down any prior run for this scenario, then render fresh.
"$0" --stop "$SCENARIO" 2>/dev/null || true
rm -rf "$SCRATCH"
mkdir -p "$SCRATCH"

# --- 1) render -------------------------------------------------------------
echo "↻  rendering…"
"$BIN" bake --config "$RECIPE" --output "$SCRATCH" --no-vcs --force --offline

PROJ_DIR=$(find "$SCRATCH" -maxdepth 1 -mindepth 1 -type d | head -1)
[[ -n "$PROJ_DIR" ]] || { echo "✘  no rendered project dir" >&2; exit 1; }
[[ -f "$PROJ_DIR/compose.local.yml" ]] || {
    echo "✘  recipe did not render compose.local.yml (container_setup is 'none'?)" >&2
    exit 1
}

# --- 2) write a minimal .env so the stack boots -----------------------------
cat > "$PROJ_DIR/.env" <<EOF
DJANGO_SETTINGS_MODULE=config.settings.local
DJANGO_SECRET_KEY=$(python3 -c "import secrets; print(secrets.token_urlsafe(48))")
DJANGO_DEBUG=True
DJANGO_ALLOWED_HOSTS=localhost,127.0.0.1,django
USE_DEBUG_TOOLBAR=False
STAFF_MFA_REQUIRED=False
PWNED_PASSWORDS_ENABLED=False
POSTGRES_USER=postgres
POSTGRES_PASSWORD=e2e-only-not-secret
POSTGRES_DB=bakery_e2e
DATABASE_URL=postgres://postgres:e2e-only-not-secret@postgres:5432/bakery_e2e
CELERY_BROKER_URL=redis://redis:6379/0
EOF

# --- 3) docker compose up --build -d ----------------------------------------
# Build a harness-only override that asks Docker to assign free ephemeral host
# ports so this e2e doesn't collide with whatever Postgres/MySQL is already
# running on the host. Written as `compose.e2e.yml` (NOT `compose.override.yml`,
# which docker compose auto-loads — that filename would leak the harness's
# port-remap into any later `docker compose up` the user runs in the rendered
# project tree). Pass it explicitly via `-f` instead.
#
# Only include entries for services the base compose actually declares —
# compose errors with "service has neither image nor build context" if the
# override names a service that doesn't exist in the base.
{
    echo 'services:'
    echo '  django:'
    echo '    ports: !override'
    echo '      - "127.0.0.1::8000"'
    if grep -qE '^  postgres:' "$PROJ_DIR/compose.local.yml"; then
        printf '  postgres:\n    ports: !override\n      - "127.0.0.1::5432"\n'
    fi
    if grep -qE '^  mysql:' "$PROJ_DIR/compose.local.yml"; then
        printf '  mysql:\n    ports: !override\n      - "127.0.0.1::3306"\n'
    fi
    if grep -qE '^  mariadb:' "$PROJ_DIR/compose.local.yml"; then
        printf '  mariadb:\n    ports: !override\n      - "127.0.0.1::3306"\n'
    fi
} > "$PROJ_DIR/compose.e2e.yml"

echo "↻  docker compose up --build -d  (first build can take 4-6 min — uv sync + apt + pnpm)…"
(cd "$PROJ_DIR" && docker compose -f compose.local.yml -f compose.e2e.yml up --build -d) > "$LOGS/compose-up.log" 2>&1 \
    || { echo "✘  docker compose up failed — see $LOGS/compose-up.log"; tail -30 "$LOGS/compose-up.log" >&2; exit 1; }

# Read the ephemeral host port Docker assigned to django:8000.
DJANGO_PORT=$(cd "$PROJ_DIR" && docker compose -f compose.local.yml -f compose.e2e.yml port django 8000 2>/dev/null | sed 's/.*://')
[[ -n "$DJANGO_PORT" ]] || { echo "✘  could not read assigned django host port" >&2; exit 1; }
echo "    Django assigned host port: $DJANGO_PORT"

# --- 4) wait for django service ---------------------------------------------
echo -n "↻  waiting for django on host :$DJANGO_PORT"
for _ in $(seq 1 90); do
    if curl -s -o /dev/null -w '%{http_code}' --max-time 2 -H "Host: localhost" "http://127.0.0.1:$DJANGO_PORT/healthz/" 2>/dev/null | grep -q 200; then
        echo " ✓"
        break
    fi
    sleep 2
    echo -n "."
done

# --- 5) verify endpoints  ---------------------------------------------------
# Use `Host: localhost` because multi-tenant recipes route requests by the
# Host header (TenantMainMiddleware → Domain lookup). For non-multi-tenant
# recipes Django happily accepts any Host in ALLOWED_HOSTS, so this is a
# no-op there but the right hostname for tenant routing.
healthz_code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 5 -H "Host: localhost" "http://127.0.0.1:$DJANGO_PORT/healthz/" 2>/dev/null || echo "000")
home_code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 5 -H "Host: localhost" "http://127.0.0.1:$DJANGO_PORT/" 2>/dev/null || echo "000")

# --- 6) capture compose ps for the log
(cd "$PROJ_DIR" && docker compose -f compose.local.yml ps) > "$LOGS/compose-ps.log" 2>&1 || true

echo
echo "===================================================================="
if [[ "$healthz_code" == "200" && "$home_code" =~ ^(200|302)$ ]]; then
    echo "✔  compose stack healthy"
else
    echo "✘  compose stack unhealthy — healthz=$healthz_code home=$home_code"
    (cd "$PROJ_DIR" && docker compose -f compose.local.yml logs --tail=20 django) 2>&1 | head -40 >&2
fi
echo "    healthz: $healthz_code"
echo "    home:    $home_code"
echo "    Logs:    $LOGS/"
echo "    Teardown: $0 --stop $SCENARIO"
echo "===================================================================="
[[ "$healthz_code" == "200" ]] || exit 1
