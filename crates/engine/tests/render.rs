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
