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
    r.frontend_variant = django_bakery_engine::FrontendVariant::Full;
    r.radix_flavor = Some(django_bakery_engine::RadixFlavor::Themes);
    r.js_language = django_bakery_engine::JsLanguage::Typescript;
    r.js_testing = true;
    r
}

fn vue_full_recipe() -> Recipe {
    let mut r = Recipe::defaults();
    r.project_slug = "vue_app".into();
    r.frontend = Frontend::Vue;
    r.frontend_variant = django_bakery_engine::FrontendVariant::Full;
    r.radix_flavor = None;
    r.js_language = django_bakery_engine::JsLanguage::Typescript;
    r.js_testing = true;
    r
}

fn next_full_recipe() -> Recipe {
    let mut r = Recipe::defaults();
    r.project_slug = "next_app".into();
    r.frontend = Frontend::Next;
    r.frontend_variant = django_bakery_engine::FrontendVariant::Full;
    r.radix_flavor = None;
    r.js_language = django_bakery_engine::JsLanguage::Typescript;
    r.js_testing = true;
    r
}

#[test]
fn next_full_recipe_emits_full_tree() {
    let (_tmp, root) = render_recipe(&next_full_recipe());
    assert_present(&root, "pnpm-workspace.yaml");
    for f in [
        "frontend/package.json",
        "frontend/tsconfig.json",
        "frontend/next.config.ts",
        "frontend/next-env.d.ts",
        "frontend/eslint.config.js",
        "frontend/.prettierrc",
        "frontend/.gitignore",
        "frontend/.env.example",
        "frontend/README.md",
        "frontend/playwright.config.ts",
        "frontend/vitest.config.ts",
        "frontend/styles/globals.css",
        "frontend/app/layout.tsx",
        "frontend/app/providers.tsx",
        "frontend/app/not-found.tsx",
        "frontend/app/page.tsx",
        "frontend/app/about/page.tsx",
        "frontend/app/account/layout.tsx",
        "frontend/app/account/login/page.tsx",
        "frontend/app/account/signup/page.tsx",
        "frontend/app/account/profile/page.tsx",
        "frontend/app/account/verify-email/page.tsx",
        "frontend/app/account/mfa-challenge/page.tsx",
        "frontend/app/account/mfa-activate/page.tsx",
        "frontend/app/account/recovery-codes/page.tsx",
        "frontend/lib/auth/client.ts",
        "frontend/lib/auth/server.ts",
        "frontend/lib/auth/csrf.ts",
        "frontend/lib/auth/store.ts",
        "frontend/lib/auth/types.ts",
        "frontend/lib/auth/tests/client.test.ts",
        "frontend/lib/api/client.ts",
        "frontend/lib/ui/nav.tsx",
        "frontend/lib/ui/theme.tsx",
        "frontend/tests/setup.ts",
        "frontend/tests/e2e/login.spec.ts",
    ] {
        assert_present(&root, f);
    }
}

#[test]
fn next_full_recipe_carries_no_skip_markers() {
    let (_tmp, root) = render_recipe(&next_full_recipe());
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
        "Next full files contain __SKIP__:\n  - {}",
        offenders.join("\n  - ")
    );
}

#[test]
fn next_full_server_auth_is_marked_server_only() {
    let (_tmp, root) = render_recipe(&next_full_recipe());
    let body = read(&root, "frontend/lib/auth/server.ts");
    assert!(
        body.contains("\"server-only\"") || body.contains("'server-only'"),
        "lib/auth/server.ts must import 'server-only' so the cookie-reading code never bundles to the browser"
    );
    assert!(
        body.contains("cookies()") || body.contains("from \"next/headers\""),
        "server.ts must use next/headers cookies() to read the session"
    );
    assert!(
        body.contains("cache: \"no-store\""),
        "server.ts auth fetch must opt out of the Next data cache (per-request user data)"
    );
}

#[test]
fn next_full_auth_client_carries_security_invariants() {
    let (_tmp, root) = render_recipe(&next_full_recipe());
    let body = read(&root, "frontend/lib/auth/client.ts");
    assert!(body.contains("credentials: \"include\""));
    assert!(body.contains("X-CSRFToken"));
    assert!(body.contains("mfa_required"));
    assert!(body.contains("email_verification_required"));
    for forbidden in [
        "localStorage.setItem",
        "localStorage.getItem",
        "sessionStorage.setItem",
        "sessionStorage.getItem",
    ] {
        assert!(!body.contains(forbidden), "Next auth client must not call {forbidden}");
    }
}

#[test]
fn next_full_zustand_store_does_not_persist_to_localstorage() {
    let (_tmp, root) = render_recipe(&next_full_recipe());
    let body = read(&root, "frontend/lib/auth/store.ts");
    for forbidden in [
        "localStorage.setItem",
        "localStorage.getItem",
        "sessionStorage.setItem",
        "sessionStorage.getItem",
        "zustand/middleware/persist",
    ] {
        assert!(
            !body.contains(forbidden),
            "Next auth store must NOT persist via {forbidden} (session cookies only)"
        );
    }
}

#[test]
fn next_full_profile_page_is_server_component_with_redirect_guard() {
    let (_tmp, root) = render_recipe(&next_full_recipe());
    let body = read(&root, "frontend/app/account/profile/page.tsx");
    assert!(
        !body.contains("\"use client\""),
        "profile page must remain a Server Component (server-side redirect gate)"
    );
    assert!(
        body.contains("redirect(\"/account/login"),
        "profile page must redirect unauthenticated users server-side"
    );
    assert!(body.contains("currentUser()"), "profile page must read currentUser() from server.ts");
}

#[test]
fn next_full_next_config_ships_security_headers() {
    let (_tmp, root) = render_recipe(&next_full_recipe());
    let body = read(&root, "frontend/next.config.ts");
    assert!(body.contains("X-Frame-Options"));
    assert!(body.contains("DENY"));
    assert!(body.contains("Permissions-Policy"));
    assert!(body.contains("camera=()"));
    assert!(body.contains("productionBrowserSourceMaps: false"));
    assert!(body.contains("/api/:path*"));
    assert!(body.contains("/_allauth/:path*"));
}

#[test]
fn next_full_eslint_bans_raw_html() {
    let (_tmp, root) = render_recipe(&next_full_recipe());
    let body = read(&root, "frontend/eslint.config.js");
    assert!(body.contains("react/no-danger"), "ESLint must ban the raw-HTML React API (OWASP A03)");
}

#[test]
fn vue_full_recipe_emits_full_tree() {
    let (_tmp, root) = render_recipe(&vue_full_recipe());
    assert_present(&root, "pnpm-workspace.yaml");
    for f in [
        "frontend/package.json",
        "frontend/tsconfig.json",
        "frontend/tsconfig.node.json",
        "frontend/vite.config.ts",
        "frontend/vitest.config.ts",
        "frontend/playwright.config.ts",
        "frontend/eslint.config.js",
        "frontend/.prettierrc",
        "frontend/.gitignore",
        "frontend/.env.example",
        "frontend/env.d.ts",
        "frontend/index.html",
        "frontend/README.md",
        "frontend/src/main.ts",
        "frontend/src/App.vue",
        "frontend/src/router/index.ts",
        "frontend/src/layouts/DefaultLayout.vue",
        "frontend/src/layouts/AccountLayout.vue",
        "frontend/src/views/HomeView.vue",
        "frontend/src/views/AboutView.vue",
        "frontend/src/views/NotFoundView.vue",
        "frontend/src/views/account/LoginView.vue",
        "frontend/src/views/account/SignupView.vue",
        "frontend/src/views/account/ProfileView.vue",
        "frontend/src/views/account/VerifyEmailView.vue",
        "frontend/src/views/account/MfaChallengeView.vue",
        "frontend/src/views/account/MfaActivateView.vue",
        "frontend/src/views/account/RecoveryCodesView.vue",
        "frontend/src/auth/client.ts",
        "frontend/src/auth/csrf.ts",
        "frontend/src/auth/types.ts",
        "frontend/src/auth/guards.ts",
        "frontend/src/stores/auth.ts",
        "frontend/src/composables/useColorMode.ts",
        "frontend/src/api/client.ts",
        "frontend/src/assets/css/main.css",
        "frontend/tests/stores/auth.test.ts",
        "frontend/tests/e2e/login.spec.ts",
    ] {
        assert_present(&root, f);
    }
}

#[test]
fn vue_full_recipe_carries_no_skip_markers() {
    let (_tmp, root) = render_recipe(&vue_full_recipe());
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
        "Vue full files contain __SKIP__:\n  - {}",
        offenders.join("\n  - ")
    );
}

#[test]
fn vue_full_auth_client_carries_security_invariants() {
    let (_tmp, root) = render_recipe(&vue_full_recipe());
    let body = read(&root, "frontend/src/auth/client.ts");
    assert!(body.contains("credentials: \"include\""));
    assert!(body.contains("X-CSRFToken"));
    assert!(body.contains("mfa_required"));
    assert!(body.contains("email_verification_required"));
    for forbidden in [
        "localStorage.setItem",
        "localStorage.getItem",
        "sessionStorage.setItem",
        "sessionStorage.getItem",
    ] {
        assert!(!body.contains(forbidden), "Vue auth client must not call {forbidden}");
    }
}

#[test]
fn vue_full_pinia_store_does_not_persist_to_localstorage() {
    let (_tmp, root) = render_recipe(&vue_full_recipe());
    let body = read(&root, "frontend/src/stores/auth.ts");
    for forbidden in [
        "localStorage.setItem",
        "localStorage.getItem",
        "sessionStorage.setItem",
        "sessionStorage.getItem",
        "pinia-plugin-persistedstate",
    ] {
        assert!(
            !body.contains(forbidden),
            "Vue Pinia store must NOT persist via {forbidden} (session cookies only)"
        );
    }
}

#[test]
fn vue_full_eslint_bans_v_html() {
    let (_tmp, root) = render_recipe(&vue_full_recipe());
    let body = read(&root, "frontend/eslint.config.js");
    assert!(body.contains("vue/no-v-html"), "ESLint must ban v-html (OWASP A03)");
}

#[test]
fn vue_full_router_has_auth_guards() {
    let (_tmp, root) = render_recipe(&vue_full_recipe());
    let guards = read(&root, "frontend/src/auth/guards.ts");
    assert!(guards.contains("router.beforeEach"), "router-level beforeEach guard required");
    assert!(guards.contains("requiresAuth"), "must check requiresAuth meta");
    assert!(guards.contains("guest"), "must check guest meta");
    let router = read(&root, "frontend/src/router/index.ts");
    assert!(router.contains("installAuthGuards"), "router/index.ts must install guards");
    assert!(
        router.contains("requiresAuth: true"),
        "auth-required routes must be annotated"
    );
}

fn nuxt_full_recipe() -> Recipe {
    let mut r = Recipe::defaults();
    r.project_slug = "nuxt_app".into();
    r.frontend = Frontend::Nuxt;
    r.frontend_variant = django_bakery_engine::FrontendVariant::Full;
    r.radix_flavor = None;
    r.js_language = django_bakery_engine::JsLanguage::Typescript;
    r.js_testing = true;
    r
}

#[test]
fn nuxt_full_recipe_emits_full_tree() {
    let (_tmp, root) = render_recipe(&nuxt_full_recipe());
    assert_present(&root, "pnpm-workspace.yaml");
    for f in [
        "frontend/package.json",
        "frontend/tsconfig.json",
        "frontend/nuxt.config.ts",
        "frontend/app.vue",
        "frontend/eslint.config.js",
        "frontend/.prettierrc",
        "frontend/.gitignore",
        "frontend/.env.example",
        "frontend/README.md",
        "frontend/playwright.config.ts",
        "frontend/vitest.config.ts",
        "frontend/app/assets/css/main.css",
        "frontend/app/layouts/default.vue",
        "frontend/app/layouts/account.vue",
        "frontend/app/pages/index.vue",
        "frontend/app/pages/about.vue",
        "frontend/app/pages/account/login.vue",
        "frontend/app/pages/account/signup.vue",
        "frontend/app/pages/account/profile.vue",
        "frontend/app/pages/account/verify-email.vue",
        "frontend/app/pages/account/mfa-challenge.vue",
        "frontend/app/pages/account/mfa-activate.vue",
        "frontend/app/pages/account/recovery-codes.vue",
        "frontend/app/composables/useAuth.ts",
        "frontend/app/composables/useCsrf.ts",
        "frontend/app/composables/useApi.ts",
        "frontend/app/middleware/auth.global.ts",
        "frontend/app/middleware/auth.ts",
        "frontend/app/middleware/guest.ts",
        "frontend/app/types/allauth.d.ts",
        "frontend/app/stores/auth.ts",
        "frontend/server/.gitkeep",
        "frontend/tests/stores/auth.test.ts",
        "frontend/tests/e2e/login.spec.ts",
    ] {
        assert_present(&root, f);
    }
}

#[test]
fn nuxt_full_recipe_carries_no_skip_markers() {
    let (_tmp, root) = render_recipe(&nuxt_full_recipe());
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
        "Nuxt full files contain __SKIP__:\n  - {}",
        offenders.join("\n  - ")
    );
}

#[test]
fn nuxt_full_auth_composable_carries_security_invariants() {
    let (_tmp, root) = render_recipe(&nuxt_full_recipe());
    let body = read(&root, "frontend/app/composables/useAuth.ts");
    assert!(body.contains("credentials: \"include\""));
    assert!(body.contains("X-CSRFToken"));
    assert!(body.contains("mfa_required"));
    assert!(body.contains("email_verification_required"));
    for forbidden in [
        "localStorage.setItem",
        "localStorage.getItem",
        "sessionStorage.setItem",
        "sessionStorage.getItem",
    ] {
        assert!(!body.contains(forbidden), "Nuxt useAuth must not call {forbidden}");
    }
}

#[test]
fn nuxt_full_pinia_store_does_not_persist_to_localstorage() {
    let (_tmp, root) = render_recipe(&nuxt_full_recipe());
    let body = read(&root, "frontend/app/stores/auth.ts");
    // Reject ACTUAL persistence — calls or imports. Comments mentioning the
    // anti-pattern are fine (and present, deliberately).
    for forbidden in [
        "localStorage.setItem",
        "localStorage.getItem",
        "sessionStorage.setItem",
        "sessionStorage.getItem",
        "window.localStorage",
        "window.sessionStorage",
        "pinia-plugin-persistedstate",
    ] {
        assert!(
            !body.contains(forbidden),
            "Pinia auth store must NOT persist via {forbidden} (session cookies only)"
        );
    }
}

#[test]
fn nuxt_full_eslint_bans_v_html() {
    let (_tmp, root) = render_recipe(&nuxt_full_recipe());
    let body = read(&root, "frontend/eslint.config.js");
    assert!(body.contains("vue/no-v-html"), "ESLint must ban v-html (OWASP A03)");
}

fn skeleton_recipe(frontend: Frontend) -> Recipe {
    let mut r = Recipe::defaults();
    r.project_slug = format!("{}_skel", frontend.as_str().replace('-', "_"));
    r.frontend = frontend;
    r.frontend_variant = django_bakery_engine::FrontendVariant::Skeleton;
    r.radix_flavor = None;
    r.js_language = django_bakery_engine::JsLanguage::Typescript;
    r.js_testing = false;
    r
}

#[test]
fn skeleton_react_emits_minimal_tree() {
    let (_tmp, root) = render_recipe(&skeleton_recipe(Frontend::React));
    for f in [
        "pnpm-workspace.yaml",
        "frontend/package.json",
        "frontend/tsconfig.json",
        "frontend/vite.config.ts",
        "frontend/eslint.config.js",
        "frontend/index.html",
        "frontend/README.md",
        "frontend/.env.example",
        "frontend/.gitignore",
        "frontend/src/main.tsx",
        "frontend/src/App.tsx",
    ] {
        assert_present(&root, f);
    }
    // The FULL-only files must NOT appear in the skeleton output.
    for f in [
        "frontend/src/router.tsx",
        "frontend/src/auth/client.ts",
        "frontend/src/routes/account/login.tsx",
        "frontend/playwright.config.ts",
    ] {
        assert_absent(&root, f);
    }
}

#[test]
fn skeleton_nuxt_emits_minimal_tree() {
    let (_tmp, root) = render_recipe(&skeleton_recipe(Frontend::Nuxt));
    for f in [
        "pnpm-workspace.yaml",
        "frontend/package.json",
        "frontend/nuxt.config.ts",
        "frontend/app.vue",
        "frontend/tsconfig.json",
        "frontend/README.md",
        "frontend/.env.example",
        "frontend/.gitignore",
        "frontend/eslint.config.js",
    ] {
        assert_present(&root, f);
    }
}

#[test]
fn skeleton_vue_emits_minimal_tree() {
    let (_tmp, root) = render_recipe(&skeleton_recipe(Frontend::Vue));
    for f in [
        "pnpm-workspace.yaml",
        "frontend/package.json",
        "frontend/vite.config.ts",
        "frontend/tsconfig.json",
        "frontend/env.d.ts",
        "frontend/index.html",
        "frontend/src/main.ts",
        "frontend/src/App.vue",
        "frontend/eslint.config.js",
        "frontend/README.md",
    ] {
        assert_present(&root, f);
    }
}

#[test]
fn skeleton_next_emits_minimal_tree() {
    let (_tmp, root) = render_recipe(&skeleton_recipe(Frontend::Next));
    for f in [
        "pnpm-workspace.yaml",
        "frontend/package.json",
        "frontend/next.config.ts",
        "frontend/tsconfig.json",
        "frontend/next-env.d.ts",
        "frontend/app/layout.tsx",
        "frontend/app/page.tsx",
        "frontend/eslint.config.js",
        "frontend/README.md",
    ] {
        assert_present(&root, f);
    }
}

#[test]
fn all_skeleton_readmes_document_owasp_baseline() {
    for fe in [Frontend::React, Frontend::Nuxt, Frontend::Vue, Frontend::Next] {
        let (_tmp, root) = render_recipe(&skeleton_recipe(fe));
        let body = read(&root, "frontend/README.md");
        assert!(
            body.contains("OWASP"),
            "{} skeleton README must document the OWASP baseline",
            fe.as_str()
        );
        let lower = body.to_lowercase();
        assert!(
            lower.contains("localstorage") && (lower.contains("never") || lower.contains("anti-pattern")),
            "{} skeleton README must warn against JWT-in-localStorage",
            fe.as_str()
        );
    }
}

#[test]
fn skeleton_eslint_bans_raw_html_injection() {
    let (_tmp, react) = render_recipe(&skeleton_recipe(Frontend::React));
    assert!(read(&react, "frontend/eslint.config.js").contains("react/no-danger"));

    let (_tmp2, vue) = render_recipe(&skeleton_recipe(Frontend::Vue));
    assert!(read(&vue, "frontend/eslint.config.js").contains("vue/no-v-html"));

    let (_tmp3, nuxt) = render_recipe(&skeleton_recipe(Frontend::Nuxt));
    assert!(read(&nuxt, "frontend/eslint.config.js").contains("vue/no-v-html"));

    let (_tmp4, next) = render_recipe(&skeleton_recipe(Frontend::Next));
    assert!(read(&next, "frontend/eslint.config.js").contains("react/no-danger"));
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
fn pnpm_workspace_yaml_present_when_pnpm_needed() {
    // SPA recipes: workspace.yaml declares `frontend/` as a package + carries
    // the project-wide pnpm 11 settings (onlyBuiltDependencies, etc.).
    let (_tmp1, r1) = render_recipe(&react_recipe());
    let body = read(&r1, "pnpm-workspace.yaml");
    assert!(body.contains("packages:"), "SPA pnpm-workspace.yaml must declare a package");
    assert!(body.contains("frontend"), "SPA pnpm-workspace.yaml must list frontend");
    assert!(body.contains("onlyBuiltDependencies"), "must carry build-script allowlist");

    // HTMX + Tailwind (the default) ships a root package.json for the Tailwind
    // CLI build — pnpm 11 requires `packages:` even when the workspace has
    // only one member, so we list "." (the project root itself).
    let mut htmx = Recipe::defaults();
    htmx.project_slug = "htmx_app".into();
    let (_tmp2, r2) = render_recipe(&htmx);
    let body = read(&r2, "pnpm-workspace.yaml");
    assert!(body.contains("packages:"), "htmx workspace.yaml must declare packages (pnpm 11 requires it)");
    assert!(body.contains("\".\""), "htmx workspace.yaml lists \".\" as the only package");
    assert!(!body.contains("- frontend"), "htmx workspace must not reference a nested frontend dir");
    assert!(body.contains("onlyBuiltDependencies"), "must still carry build-script allowlist");

    // HTMX without Tailwind: no Node at all, so no workspace.yaml.
    let mut htmx_no_css = Recipe::defaults();
    htmx_no_css.project_slug = "htmx_bare".into();
    htmx_no_css.css_framework = django_bakery_engine::CssFramework::None;
    let (_tmp3, r3) = render_recipe(&htmx_no_css);
    assert_absent(&r3, "pnpm-workspace.yaml");
}

#[test]
fn github_actions_workflow_renders_with_expected_shape() {
    // CI = github-actions ships TWO workflow files. Both must render to a
    // shape an Actions runner will accept: name + on + jobs at top level.
    let mut r = Recipe::defaults();
    r.project_slug = "ci_test".into();
    r.ci = django_bakery_engine::CiProvider::GitHubActions;
    let (_tmp, root) = render_recipe(&r);

    let ci = read(&root, ".github/workflows/ci.yml");
    for sentinel in ["name: CI", "on:", "jobs:", "lint-and-test:", "runs-on:"] {
        assert!(ci.contains(sentinel), "ci.yml missing sentinel: {sentinel}");
    }
    // Generated projects use uv for Python and pnpm for JS — the workflow
    // should reflect that, not pip / npm / poetry.
    assert!(ci.contains("uv"), "ci.yml should drive python via uv");

    let deploy = read(&root, ".github/workflows/deploy.yml");
    for sentinel in ["name: Deploy", "on:", "jobs:", "deploy:", "tags:"] {
        assert!(deploy.contains(sentinel), "deploy.yml missing sentinel: {sentinel}");
    }
}

#[test]
fn compose_traefik_setup_ships_both_compose_files_and_traefik_config() {
    let mut r = Recipe::defaults();
    r.project_slug = "compose_test".into();
    r.container_setup = django_bakery_engine::ContainerSetup::ComposeTraefik;
    let (_tmp, root) = render_recipe(&r);

    // Local + production compose files
    for f in ["compose.local.yml", "compose.production.yml"] {
        let body = read(&root, f);
        assert!(body.contains("services:"), "{f} must declare services");
        assert!(body.contains("django:"), "{f} must include the django service");
    }

    // Production compose adds traefik
    let prod = read(&root, "compose.production.yml");
    assert!(prod.contains("traefik:"), "production compose must include traefik service");

    // Traefik config + Dockerfiles exist
    let traefik_yml = read(&root, "compose/production/traefik/traefik.yml");
    assert!(traefik_yml.contains("entryPoints:"), "traefik.yml must define entryPoints");
    assert!(traefik_yml.contains("certificatesResolvers:") || traefik_yml.contains("certificates"),
            "traefik.yml must configure TLS resolver");
}

#[test]
fn compose_only_setup_ships_compose_files_without_traefik() {
    let mut r = Recipe::defaults();
    r.project_slug = "compose_only".into();
    r.container_setup = django_bakery_engine::ContainerSetup::ComposeOnly;
    let (_tmp, root) = render_recipe(&r);

    let prod = read(&root, "compose.production.yml");
    assert!(!prod.contains("traefik:"), "compose-only must NOT include traefik");
    assert_absent(&root, "compose/production/traefik/traefik.yml");
}

#[test]
fn no_container_setup_ships_no_compose_files() {
    let mut r = Recipe::defaults();
    r.project_slug = "no_container".into();
    r.container_setup = django_bakery_engine::ContainerSetup::None;
    let (_tmp, root) = render_recipe(&r);
    assert_absent(&root, "compose.local.yml");
    assert_absent(&root, "compose.production.yml");
    assert_absent(&root, "compose/production/django/Dockerfile");
}

#[test]
fn ci_workflow_absent_when_provider_is_none() {
    let mut r = Recipe::defaults();
    r.project_slug = "no_ci".into();
    r.ci = django_bakery_engine::CiProvider::None;
    let (_tmp, root) = render_recipe(&r);
    assert_absent(&root, ".github/workflows/ci.yml");
    assert_absent(&root, ".github/workflows/deploy.yml");
    assert_absent(&root, ".gitlab-ci.yml");
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
