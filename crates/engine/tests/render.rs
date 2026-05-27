//! Integration tests for the full render pipeline.
//!
//! These render the embedded template tree to a tempdir and assert on the file structure
//! + content. Run with `cargo test -p django-bakery-engine --test render`.

use std::fs;
use std::path::Path;

use django_bakery_engine::{
    ApiLayer, CiProvider, ContainerSetup, CssFramework, Frontend, GraphDb, ProdEmail, Recipe,
    RelationalDb, RenderOptions, ResolveMode, Storage, TypeChecker, render,
};
use tempfile::TempDir;

fn render_recipe(recipe: &Recipe) -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("create tempdir");
    let mut opts = RenderOptions::new(tmp.path());
    opts.run_hooks = false;
    opts.git_init = false;
    opts.bootstrap = false;
    opts.version_mode = ResolveMode::Offline;
    let report = render(recipe, &opts).expect("render must succeed");
    let project_root = report.project_root.clone();
    (tmp, project_root)
}

fn minimal_recipe() -> Recipe {
    let mut r = Recipe::defaults();
    r.project_slug = "minimal_app".into();
    r.relational_db = RelationalDb::Sqlite;
    r.api_layer = ApiLayer::None;
    r.frontend = Frontend::None;
    r.css_framework = CssFramework::None;
    r.use_celery = false;
    r.use_mailpit = false;
    r.use_sentry = false;
    r.use_observability = false;
    r.use_pre_commit = false;
    r.use_feature_flags = false;
    r.graph_db = GraphDb::None;
    r.storage = Storage::Whitenoise;
    r.prod_email = ProdEmail::Console;
    r.type_checker = TypeChecker::None;
    r.container_setup = ContainerSetup::None;
    r.ci = CiProvider::None;
    r
}

fn assert_present(root: &Path, rel: &str) {
    assert!(
        root.join(rel).exists(),
        "expected {rel} to exist, missing under {}",
        root.display()
    );
}

fn assert_absent(root: &Path, rel: &str) {
    assert!(
        !root.join(rel).exists(),
        "{rel} should NOT exist under {}",
        root.display()
    );
}

fn read(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel))
        .unwrap_or_else(|_| panic!("failed to read {rel} under {}", root.display()))
}

#[test]
fn default_recipe_renders_full_stack() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);

    // Core scaffolding — must exist for any project.
    for f in [
        "manage.py",
        "pyproject.toml",
        "README.md",
        "LICENSE",
        ".env.example",
        ".gitignore",
        ".editorconfig",
        ".dockerignore",
        "justfile",
        "config/__init__.py",
        "config/settings/__init__.py",
        "config/settings/base.py",
        "config/settings/local.py",
        "config/settings/production.py",
        "config/settings/test.py",
        "config/urls.py",
        "config/asgi.py",
        "config/wsgi.py",
        "apps/__init__.py",
        "apps/users/__init__.py",
        "apps/users/models.py",
        "apps/users/managers.py",
        "apps/users/admin.py",
        "apps/core/views.py",
        "templates/base.html",
        "templates/pages/home.html",
        "static/css/app.css",
    ] {
        assert_present(&root, f);
    }

    // Default recipe has Docker + Traefik + Celery + GH Actions + pre-commit + Ninja API.
    for f in [
        "compose.local.yml",
        "compose.production.yml",
        "compose/local/django/Dockerfile",
        "compose/local/django/start",
        "compose/production/django/Dockerfile",
        "compose/production/django/start",
        "compose/production/django/entrypoint",
        "compose/production/django/celery/worker/start",
        "compose/production/traefik/Dockerfile",
        "compose/production/traefik/traefik.yml",
        "compose/production/postgres/Dockerfile",
        ".github/workflows/ci.yml",
        ".github/workflows/deploy.yml",
        ".pre-commit-config.yaml",
        "config/celery_app.py",
        "apps/api/__init__.py",
        "apps/api/ninja_api.py",
    ] {
        assert_present(&root, f);
    }

    // The two DRF / GraphQL alternates must NOT be emitted for an api_layer=ninja recipe.
    assert_absent(&root, "apps/api/serializers.py");
    assert_absent(&root, "apps/api/viewsets.py");
    assert_absent(&root, "apps/api/schema.py");
}

#[test]
fn minimal_recipe_skips_optional_artifacts() {
    let recipe = minimal_recipe();
    let (_tmp, root) = render_recipe(&recipe);

    // Core scaffolding survives.
    assert_present(&root, "manage.py");
    assert_present(&root, "pyproject.toml");
    assert_present(&root, ".env.example");
    assert_present(&root, "config/settings/base.py");
    assert_present(&root, "apps/users/models.py");

    // Everything toggled OFF must be absent.
    for f in [
        "compose.local.yml",
        "compose.production.yml",
        "compose",
        ".github",
        ".pre-commit-config.yaml",
        "config/celery_app.py",
        "apps/api",
    ] {
        assert_absent(&root, f);
    }
}

#[test]
fn dotfile_shadow_convention_emits_real_dotfiles() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);

    // The `_dot_X` source files must come out as real dotfiles.
    for d in [".env.example", ".gitignore", ".editorconfig", ".dockerignore"] {
        assert_present(&root, d);
    }
    assert_present(&root, ".vscode/settings.json");
    assert_present(&root, ".github/workflows/ci.yml");

    // The literal `_dot_X` form must never leak through.
    for d in ["_dot_env.example", "_dot_gitignore", "_dot_vscode"] {
        assert_absent(&root, d);
    }
}

#[test]
fn rendered_files_never_contain_skip_sentinel() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);

    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&root) {
        let entry = entry.expect("walk");
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        // Don't bother sniffing binary assets.
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if matches!(
            ext,
            "png" | "jpg" | "jpeg" | "ico" | "woff" | "woff2" | "ttf"
        ) {
            continue;
        }
        let bytes = fs::read(path).expect("read");
        let body = String::from_utf8_lossy(&bytes);
        if body.contains("__SKIP__") {
            offenders.push(format!("{name}: {}", path.display()));
        }
    }
    assert!(
        offenders.is_empty(),
        "rendered files contain __SKIP__ marker:\n  - {}",
        offenders.join("\n  - "),
    );
}

#[test]
fn base_settings_reflects_recipe_choices() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let body = read(&root, "config/settings/base.py");

    // PostgreSQL DATABASE_URL default must appear; SQLite default must NOT.
    assert!(
        body.contains("DATABASE_URL"),
        "base.py must define DATABASE_URL"
    );
    assert!(
        body.contains("postgres://"),
        "default recipe is postgres but base.py omits the postgres scheme"
    );
    assert!(
        !body.contains("sqlite:///"),
        "default recipe is postgres but base.py still has the sqlite default"
    );
    // Auth defaults
    assert!(
        body.contains("AUTH_USER_MODEL = \"users.User\""),
        "AUTH_USER_MODEL must be wired to our custom user model"
    );
    assert!(
        body.contains("Argon2PasswordHasher"),
        "Argon2 must be the first hasher"
    );
    // OWASP defaults
    assert!(body.contains("SECURE_REFERRER_POLICY"));
    assert!(body.contains("X_FRAME_OPTIONS"));
    // allauth headless mode is gated on SPA frontends — default recipe is HTMX, so this
    // should NOT appear.
    assert!(
        !body.contains("HEADLESS_FRONTEND_URLS"),
        "default recipe is HTMX; headless allauth config should not be present"
    );
}

#[test]
fn spa_frontend_enables_headless_allauth() {
    let mut recipe = Recipe::defaults();
    recipe.frontend = Frontend::React;
    recipe.radix_flavor = Some(django_bakery_engine::RadixFlavor::Themes);
    let (_tmp, root) = render_recipe(&recipe);
    let body = read(&root, "config/settings/base.py");
    assert!(
        body.contains("allauth.headless"),
        "React frontend must install allauth.headless"
    );
    assert!(
        body.contains("HEADLESS_FRONTEND_URLS"),
        "React frontend must configure HEADLESS_FRONTEND_URLS"
    );
}

#[test]
fn pyproject_pins_chosen_api_layer() {
    let mut r = Recipe::defaults();
    r.api_layer = ApiLayer::Drf;
    let (_tmp, root) = render_recipe(&r);
    let body = read(&root, "pyproject.toml");
    assert!(body.contains("djangorestframework"));
    assert!(!body.contains("django-ninja>="));

    let mut r2 = Recipe::defaults();
    r2.project_slug = "ninja_app".into();
    r2.api_layer = ApiLayer::Ninja;
    let (_tmp2, root2) = render_recipe(&r2);
    let body2 = read(&root2, "pyproject.toml");
    assert!(body2.contains("django-ninja"));
    assert!(!body2.contains("djangorestframework>="));
}

#[test]
fn ninja_api_enforces_staff_only_user_listing() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let body = read(&root, "apps/api/ninja_api.py");
    // The /me endpoint exists and is unauthenticated to non-staff but scoped to caller.
    assert!(body.contains("@api.get(\"/me\""), "/me endpoint must exist");
    assert!(
        body.contains("await User.objects.aget(pk=request.user.pk)"),
        "/me must fetch by request.user.pk only — never by an arbitrary user_id"
    );
    // /users and /users/{id} must hard-gate on _require_staff.
    assert!(
        body.contains("_require_staff(request)"),
        "staff-only endpoints must call the _require_staff gate"
    );
    assert!(
        body.contains("not user.is_staff"),
        "the gate must check is_staff and reject anything else"
    );
    // The pre-fix vulnerable shape must NOT come back: a /users endpoint that returns
    // ALL users to any authenticated session.
    assert!(
        !body.contains("async def list_users(request, limit: int = 50, offset: int = 0) -> list[UserOut]:\n    qs = User.objects.all().order_by(\"id\")[offset : offset + limit]\n    return"),
        "vulnerable pre-fix list_users body has reappeared"
    );
}

#[test]
fn drf_user_viewset_is_admin_only() {
    let mut r = Recipe::defaults();
    r.api_layer = ApiLayer::Drf;
    let (_tmp, root) = render_recipe(&r);
    let body = read(&root, "apps/api/viewsets.py");
    assert!(
        body.contains("permission_classes = [IsAdminUser]"),
        "UserViewSet must be IsAdminUser (list/retrieve), not just IsAuthenticated"
    );
    assert!(
        body.contains("url_path=\"me\"") && body.contains("IsAuthenticated"),
        "the /me action must exist and be IsAuthenticated (not IsAdminUser)"
    );
}

#[test]
fn env_example_uses_placeholders_not_real_secrets() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let body = read(&root, ".env.example");
    // Every secret field must be a placeholder, not a CSPRNG value.
    for line_prefix in ["DJANGO_SECRET_KEY=", "POSTGRES_PASSWORD=", "REDIS_PASSWORD="] {
        let line = body
            .lines()
            .find(|l| l.starts_with(line_prefix))
            .unwrap_or_else(|| panic!("missing {line_prefix} in .env.example"));
        let value = line.trim_start_matches(line_prefix);
        assert!(
            value.starts_with("<GENERATE"),
            "{line_prefix}: secret slot must be a placeholder; got {value:?}"
        );
    }
    // Flower creds must be required (no default).
    assert!(body.contains("CELERY_FLOWER_USER=<GENERATE"));
    assert!(body.contains("CELERY_FLOWER_PASSWORD=<GENERATE"));
}

#[test]
fn production_compose_uses_env_redis_password() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let body = read(&root, "compose.production.yml");
    assert!(
        body.contains("$${REDIS_PASSWORD}"),
        "redis must read its password from the env, not have it baked in"
    );
    // Belt-and-braces: confirm we don't accidentally bake the literal value.
    assert!(
        !body.contains("--requirepass {{"),
        "Jinja shouldn't leak — should have rendered already"
    );
}

#[test]
fn base_settings_secret_key_no_insecure_default() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let base = read(&root, "config/settings/base.py");
    // The old "insecure-fallback-only-for-checks" string must not be present anymore.
    assert!(
        !base.contains("insecure-fallback-only-for-checks"),
        "base.py must NOT carry the old hardcoded fallback secret key"
    );
    assert!(
        base.contains("ImproperlyConfigured")
            && base.contains("DJANGO_SECRET_KEY must be set in production"),
        "base.py must hard-fail in production when DJANGO_SECRET_KEY is unset"
    );
}

#[test]
fn secure_proxy_ssl_header_only_in_production_settings() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let base = read(&root, "config/settings/base.py");
    let prod = read(&root, "config/settings/production.py");
    assert!(
        !base.contains("SECURE_PROXY_SSL_HEADER = "),
        "SECURE_PROXY_SSL_HEADER must NOT be ASSIGNED in base.py (spoofable in dev). \
         A comment explaining the move is fine."
    );
    assert!(
        prod.contains("SECURE_PROXY_SSL_HEADER = "),
        "production.py must set SECURE_PROXY_SSL_HEADER (deployment is behind a proxy)"
    );
}

#[test]
fn csrf_cookie_httponly_not_set_to_true() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let base = read(&root, "config/settings/base.py");
    assert!(
        !base.contains("CSRF_COOKIE_HTTPONLY = True"),
        "CSRF_COOKIE_HTTPONLY=True breaks every SPA flow — must be removed"
    );
}

#[test]
fn flower_local_start_has_no_default_credentials() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let body = read(&root, "compose/local/django/celery/flower/start");
    assert!(
        !body.contains(":-admin") && !body.contains(":-flower"),
        "Flower start script must not have hardcoded credential fallbacks"
    );
    assert!(
        body.contains("CELERY_FLOWER_USER must be set")
            && body.contains("CELERY_FLOWER_PASSWORD must be set"),
        "Flower start script must fail-fast when credentials are missing"
    );
}

#[test]
fn rate_limits_cover_signup_and_reset() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let body = read(&root, "config/settings/base.py");
    for key in ["\"login_failed\":", "\"signup\":", "\"reset_password\":"] {
        assert!(
            body.contains(key),
            "ACCOUNT_RATE_LIMITS must rate-limit {key} too, not just login_failed"
        );
    }
}

#[test]
fn pwned_password_validator_present() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let body = read(&root, "config/settings/base.py");
    assert!(
        body.contains("pwned_passwords_django.validators.PwnedPasswordsValidator"),
        "AUTH_PASSWORD_VALIDATORS must include the breached-password (HIBP) check"
    );
    let test_settings = read(&root, "config/settings/test.py");
    assert!(
        test_settings.contains("PWNED_PASSWORDS = {\"ENABLED\": False}"),
        "test settings must disable HIBP lookup so CI runs offline"
    );
}

#[test]
fn permissions_policy_middleware_wired() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let base = read(&root, "config/settings/base.py");
    assert!(
        base.contains("apps.core.middleware.PermissionsPolicyMiddleware"),
        "PermissionsPolicyMiddleware must be in MIDDLEWARE"
    );
    assert_present(&root, "apps/core/middleware.py");
    let mw = read(&root, "apps/core/middleware.py");
    for feature in ["camera", "microphone", "geolocation", "payment"] {
        let token = format!("\"{feature}\": \"()\"");
        assert!(
            mw.contains(&token),
            "default policy must deny {feature}; missing `{token}`"
        );
    }
}

#[test]
fn admin_url_has_random_suffix() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let env_example = read(&root, ".env.example");
    // The example must contain `admin-<16-lowercase-hex-ish>/` — NOT bare `admin/`.
    let line = env_example
        .lines()
        .find(|l| l.starts_with("DJANGO_ADMIN_URL="))
        .expect("DJANGO_ADMIN_URL missing from .env.example");
    let value = line.trim_start_matches("DJANGO_ADMIN_URL=");
    assert!(
        value.starts_with("admin-") && value.ends_with("/") && value.len() > "admin-/".len() + 8,
        "DJANGO_ADMIN_URL must be a random-suffixed path, got {value:?}"
    );
    assert_ne!(value, "admin/", "the unguessable default is the whole point");
}

#[test]
fn session_lifetime_is_explicit() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let base = read(&root, "config/settings/base.py");
    let prod = read(&root, "config/settings/production.py");
    assert!(
        base.contains("ACCOUNT_SESSION_REMEMBER = None"),
        "remember-me must be explicit (None), not auto-on (True)"
    );
    assert!(
        base.contains("SESSION_SAVE_EVERY_REQUEST = True"),
        "active sessions must slide forward"
    );
    assert!(
        prod.contains("SESSION_COOKIE_AGE = 60 * 60 * 24\n"),
        "production session lifetime should be 1 day, not 14"
    );
}

#[test]
fn auth_signals_wired_for_audit_logging() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    assert_present(&root, "apps/users/signals.py");
    let signals = read(&root, "apps/users/signals.py");
    assert!(signals.contains("user_login_failed"));
    assert!(signals.contains("user_logged_in"));
    assert!(signals.contains("user_signed_up"));
    let apps_cfg = read(&root, "apps/users/apps.py");
    assert!(
        apps_cfg.contains("from apps.users import signals"),
        "signals must be imported in UsersConfig.ready() to register receivers"
    );
}

#[test]
fn readyz_does_not_leak_exception_detail() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let body = read(&root, "apps/core/views.py");
    // Old form leaked: `"detail": str(exc)` — the fixed version must use `logger.exception`
    // and return a generic 503 body.
    assert!(
        !body.contains("\"detail\": str(exc)"),
        "readyz must not echo the exception detail in the HTTP body"
    );
    assert!(
        body.contains("logger.exception"),
        "readyz must log the exception server-side"
    );
}

#[test]
fn mfa_middleware_wired_in_settings() {
    let recipe = Recipe::defaults();
    let (_tmp, root) = render_recipe(&recipe);
    let base = read(&root, "config/settings/base.py");
    assert!(
        base.contains("apps.users.middleware.RequireMfaForStaffMiddleware"),
        "MFA-for-staff middleware must be wired in MIDDLEWARE"
    );
    assert!(
        base.contains("STAFF_MFA_REQUIRED"),
        "STAFF_MFA_REQUIRED setting must be exposed for env override"
    );
    // And the middleware file itself must exist.
    assert_present(&root, "apps/users/middleware.py");
    let mw = read(&root, "apps/users/middleware.py");
    assert!(
        mw.contains("is_mfa_enabled") && mw.contains("redirect(\"mfa_activate_totp\")"),
        "middleware must check is_mfa_enabled and redirect to enrollment"
    );

    // Test settings disable enforcement so factories can log in as staff in tests.
    let test_settings = read(&root, "config/settings/test.py");
    assert!(
        test_settings.contains("STAFF_MFA_REQUIRED = False"),
        "test settings must disable the MFA gate"
    );
}

#[test]
fn malicious_recipe_slug_with_traversal_is_rejected_by_validator() {
    // Validator gate — first line of defense.
    let mut r = Recipe::defaults();
    r.project_slug = "../etc/passwd".into();
    assert!(r.validate().is_err(), "validator must reject path-traversal in slug");
}

fn react_recipe() -> Recipe {
    let mut r = Recipe::defaults();
    r.project_slug = "react_app".into();
    r.frontend = Frontend::React;
    r.radix_flavor = Some(django_bakery_engine::RadixFlavor::Themes);
    r.js_language = django_bakery_engine::JsLanguage::Typescript;
    r.js_testing = true;
    r
}

#[test]
fn frontend_react_recipe_emits_full_tree() {
    let recipe = react_recipe();
    let (_tmp, root) = render_recipe(&recipe);
    assert_present(&root, "pnpm-workspace.yaml");
    for f in [
        "frontend/package.json",
        "frontend/tsconfig.json",
        "frontend/vite.config.ts",
        "frontend/vitest.config.ts",
        "frontend/playwright.config.ts",
        "frontend/eslint.config.js",
        "frontend/.prettierrc",
        "frontend/.gitignore",
        "frontend/.env.example",
        "frontend/index.html",
        "frontend/README.md",
        "frontend/src/main.tsx",
        "frontend/src/router.tsx",
        "frontend/src/env.ts",
        "frontend/src/routes/_layout.tsx",
        "frontend/src/routes/index.tsx",
        "frontend/src/routes/about.tsx",
        "frontend/src/routes/_not-found.tsx",
        "frontend/src/routes/account/login.tsx",
        "frontend/src/routes/account/signup.tsx",
        "frontend/src/routes/account/profile.tsx",
        "frontend/src/routes/account/mfa-challenge.tsx",
        "frontend/src/routes/account/mfa-activate.tsx",
        "frontend/src/routes/account/verify-email.tsx",
        "frontend/src/routes/account/recovery-codes.tsx",
        "frontend/src/auth/client.ts",
        "frontend/src/auth/csrf.ts",
        "frontend/src/auth/store.ts",
        "frontend/src/auth/guards.tsx",
        "frontend/src/auth/types.ts",
        "frontend/src/auth/tests/client.test.ts",
        "frontend/src/api/client.ts",
        "frontend/src/ui/nav.tsx",
        "frontend/src/ui/theme.tsx",
        "frontend/tests/e2e/login.spec.ts",
    ] {
        assert_present(&root, f);
    }
}

#[test]
fn frontend_recipe_carries_no_skip_markers() {
    let (_tmp, root) = render_recipe(&react_recipe());
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root.join("frontend")) {
        let entry = entry.expect("walk");
        if !entry.file_type().is_file() {
            continue;
        }
        let body = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if body.contains("__SKIP__") {
            offenders.push(entry.path().display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "frontend files contain __SKIP__ markers:\n  - {}",
        offenders.join("\n  - ")
    );
}

#[test]
fn frontend_dotfile_shadow_extends_to_subtree() {
    let (_tmp, root) = render_recipe(&react_recipe());
    for d in ["frontend/.gitignore", "frontend/.env.example", "frontend/.prettierrc"] {
        assert_present(&root, d);
    }
    for raw in ["frontend/_dot_gitignore", "frontend/_dot_env.example", "frontend/_dot_prettierrc"] {
        assert_absent(&root, raw);
    }
}

#[test]
fn pnpm_workspace_yaml_only_for_spa_recipes() {
    let (_tmp1, r1) = render_recipe(&react_recipe());
    let body = read(&r1, "pnpm-workspace.yaml");
    assert!(body.contains("frontend"), "pnpm-workspace.yaml must reference frontend");

    let mut htmx = Recipe::defaults();
    htmx.project_slug = "htmx_app".into();
    let (_tmp2, r2) = render_recipe(&htmx);
    assert_absent(&r2, "pnpm-workspace.yaml");
}

#[test]
fn csp_connect_src_extended_for_spa_origins() {
    let (_tmp, root) = render_recipe(&react_recipe());
    let body = read(&root, "config/settings/base.py");
    assert!(body.contains("http://localhost:5173"), "CSP/CORS must include the SPA dev origin");
    assert!(
        body.contains("CSP_CONNECT_SRC = (\"'self'\", \"http://localhost:5173\")"),
        "CSP_CONNECT_SRC must include the SPA origin"
    );
    assert!(
        body.contains("CSRF_TRUSTED_ORIGINS"),
        "CSRF_TRUSTED_ORIGINS must be set when SPA frontend is selected"
    );
}

#[test]
fn frontend_compose_service_added_for_spa() {
    let (_tmp, root) = render_recipe(&react_recipe());
    let body = read(&root, "compose.local.yml");
    assert!(body.contains("frontend:"), "frontend service must be in compose.local.yml");
    assert!(body.contains("node:24-alpine"), "Node 24+ alpine image expected");
    assert!(body.contains("\"5173:5173\""), "Vite dev port 5173 must be exposed");
}

#[test]
fn react_auth_client_carries_security_invariants() {
    let (_tmp, root) = render_recipe(&react_recipe());
    let body = read(&root, "frontend/src/auth/client.ts");
    assert!(body.contains("credentials: \"include\""), "every fetch must send credentials");
    assert!(body.contains("X-CSRFToken"), "auth client must forward CSRF header");
    assert!(body.contains("mfa_required"), "MFA branch must be wired");
    assert!(body.contains("email_verification_required"), "verify-email branch must be wired");
    // Reject any actual localStorage / sessionStorage CALL — a comment mentioning the
    // anti-pattern is fine (and present, deliberately).
    for forbidden in [
        "localStorage.setItem",
        "localStorage.getItem",
        "sessionStorage.setItem",
        "sessionStorage.getItem",
        "window.localStorage",
        "window.sessionStorage",
    ] {
        assert!(
            !body.contains(forbidden),
            "auth client must NOT call {forbidden} (session cookies only)"
        );
    }
}

#[test]
fn force_overwrites_existing_directory() {
    let tmp = TempDir::new().unwrap();
    let recipe = Recipe::defaults();
    let mut opts = RenderOptions::new(tmp.path());
    opts.run_hooks = false;
    opts.git_init = false;
    opts.version_mode = ResolveMode::Offline;

    render(&recipe, &opts).expect("first render");
    // Second render without --force should error
    let err = render(&recipe, &opts).expect_err("must fail without --force");
    assert!(format!("{err}").contains("already exists"));
    // With --force it succeeds.
    opts.force = true;
    render(&recipe, &opts).expect("force render");
}
