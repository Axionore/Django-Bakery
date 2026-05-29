//! Builds the rendering context — a `minijinja::Value` — from a [`Recipe`].
//!
//! Templates address everything via `bakery.*` — a single flat namespace that covers
//! both the user's prompt answers (e.g. `bakery.project_slug`, `bakery.api_layer`,
//! `bakery.multi_tenant`) and engine-computed extras (live-resolved version pins via
//! `bakery.versions['<pkg>']`, random secrets like `bakery.django_secret_key`, derived
//! booleans like `bakery.is_postgres`, `bakery.has_frontend_spa`).

use std::time::{SystemTime, UNIX_EPOCH};

use heck::{ToKebabCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use minijinja::Value;
use rand::SeedableRng;
use rand::seq::IndexedRandom;
use serde_json::{Map, json};

use crate::recipe::Recipe;

/// Computed context exposed to templates under a single `bakery` namespace.
pub struct Context;

impl Context {
    /// Build a [`minijinja::Value`] tree from a recipe with default (bundled) versions.
    pub fn build(recipe: &Recipe) -> Value {
        let versions = crate::versions::resolve(recipe, crate::versions::ResolveMode::Offline);
        Self::build_with_versions(recipe, &versions)
    }

    /// Build a context with a pre-resolved version map (the normal path during render).
    pub fn build_with_versions(recipe: &Recipe, versions: &crate::versions::VersionMap) -> Value {
        // Merge recipe fields + engine extras into one flat `bakery` map. Recipe fields
        // are inserted first so engine extras that happen to share a name (e.g. derived
        // booleans like `is_postgres`) win deterministically — by convention the extras
        // re-derive the boolean from the recipe field so the value is the same either way.
        let mut bakery = Self::recipe_fields(recipe);
        for (k, v) in Self::engine_extras(recipe) {
            bakery.insert(k, v);
        }

        let mut v_obj = serde_json::Map::new();
        for (k, val) in versions {
            // Strip the "py." / "npm." namespace prefix for clean template access.
            // e.g. `versions.py.django` → ok, `versions.django` → also ok (last writer wins).
            v_obj.insert(k.clone(), json!(val));
            if let Some(short) = k.split_once('.').map(|(_, n)| n.to_string()) {
                v_obj.insert(short, json!(val));
            }
        }
        bakery.insert("versions".into(), serde_json::Value::Object(v_obj));

        let root = json!({ "bakery": bakery });
        Value::from_serialize(&root)
    }

    /// The recipe's user-facing fields (prompt answers + derived booleans), flattened.
    fn recipe_fields(r: &Recipe) -> Map<String, serde_json::Value> {
        let mut m = Map::new();
        // basics
        m.insert("project_name".into(), json!(r.project_name));
        m.insert("project_slug".into(), json!(r.project_slug));
        m.insert(
            "project_module".into(),
            json!(r.project_slug.to_snake_case()),
        );
        m.insert(
            "project_camel".into(),
            json!(r.project_slug.to_upper_camel_case()),
        );
        m.insert("project_kebab".into(), json!(r.project_slug.to_kebab_case()));
        m.insert(
            "project_shouty".into(),
            json!(r.project_slug.to_shouty_snake_case()),
        );
        m.insert("description".into(), json!(r.description));
        m.insert("author_name".into(), json!(r.author_name));
        m.insert("author_email".into(), json!(r.author_email));
        m.insert("domain_name".into(), json!(r.domain_name));
        m.insert("license".into(), json!(r.license.as_str()));
        m.insert("open_source".into(), json!(r.open_source));
        m.insert("timezone".into(), json!(r.timezone));

        // stack
        m.insert("python_version".into(), json!(r.python_version.dotted()));
        m.insert(
            "python_version_short".into(),
            json!(r.python_version.short()),
        );
        m.insert("django_version".into(), json!(r.django_version.as_str()));
        m.insert("mode".into(), json!(r.mode.as_str()));
        m.insert("relational_db".into(), json!(r.relational_db.as_str()));
        m.insert("graph_db".into(), json!(r.graph_db.as_str()));
        m.insert("api_layer".into(), json!(r.api_layer.as_str()));
        m.insert("frontend".into(), json!(r.frontend.as_str()));
        m.insert(
            "radix_flavor".into(),
            json!(r.radix_flavor.map(|f| f.as_str()).unwrap_or("none")),
        );
        m.insert("js_language".into(), json!(r.js_language.as_str()));
        m.insert(
            "is_typescript".into(),
            json!(matches!(r.js_language, crate::recipe::JsLanguage::Typescript)),
        );
        m.insert("js_testing".into(), json!(r.js_testing));
        m.insert("css_framework".into(), json!(r.css_framework.as_str()));

        // add-ons
        m.insert("use_celery".into(), json!(r.use_celery));
        m.insert("celery_broker".into(), json!(r.celery_broker.as_str()));
        m.insert("use_mailpit".into(), json!(r.use_mailpit));
        m.insert("prod_email".into(), json!(r.prod_email.as_str()));
        m.insert("storage".into(), json!(r.storage.as_str()));
        m.insert("use_sentry".into(), json!(r.use_sentry));
        m.insert("use_observability".into(), json!(r.use_observability));
        m.insert("use_feature_flags".into(), json!(r.use_feature_flags));
        m.insert("multi_tenant".into(), json!(r.multi_tenant));
        m.insert("type_checker".into(), json!(r.type_checker.as_str()));
        m.insert("use_pre_commit".into(), json!(r.use_pre_commit));
        m.insert("ci".into(), json!(r.ci.as_str()));
        m.insert("container_setup".into(), json!(r.container_setup.as_str()));
        m.insert("version_control".into(), json!(r.version_control.as_str()));

        // Computed booleans — referenced as `bakery.is_postgres` etc. throughout the
        // templates. Eagerly derived here so templates don't have to encode the same
        // matcher expression in multiple places.
        m.insert(
            "is_postgres".into(),
            json!(matches!(r.relational_db, crate::recipe::RelationalDb::Postgres)),
        );
        m.insert(
            "is_sqlite".into(),
            json!(matches!(r.relational_db, crate::recipe::RelationalDb::Sqlite)),
        );
        m.insert(
            "is_mysqlish".into(),
            json!(matches!(
                r.relational_db,
                crate::recipe::RelationalDb::Mysql | crate::recipe::RelationalDb::Mariadb
            )),
        );
        m.insert(
            "is_mysql".into(),
            json!(matches!(r.relational_db, crate::recipe::RelationalDb::Mysql)),
        );
        m.insert(
            "is_mariadb".into(),
            json!(matches!(r.relational_db, crate::recipe::RelationalDb::Mariadb)),
        );
        m.insert(
            "has_api".into(),
            json!(!matches!(r.api_layer, crate::recipe::ApiLayer::None)),
        );
        m.insert(
            "has_frontend_spa".into(),
            json!(matches!(
                r.frontend,
                crate::recipe::Frontend::React
                    | crate::recipe::Frontend::Nuxt
                    | crate::recipe::Frontend::Vue
                    | crate::recipe::Frontend::Next
            )),
        );
        m.insert(
            "wants_docker".into(),
            json!(!matches!(
                r.container_setup,
                crate::recipe::ContainerSetup::None
            )),
        );
        m.insert(
            "wants_traefik".into(),
            json!(matches!(
                r.container_setup,
                crate::recipe::ContainerSetup::ComposeTraefik
            )),
        );
        // Frontend-context booleans + dev port.
        let dev_port: u16 = match r.frontend {
            crate::recipe::Frontend::React | crate::recipe::Frontend::Vue => 5173,
            crate::recipe::Frontend::Nuxt | crate::recipe::Frontend::Next => 3000,
            _ => 0,
        };
        m.insert("frontend_dev_port".into(), json!(dev_port));
        m.insert(
            "frontend_origin".into(),
            json!(format!("http://localhost:{}", dev_port)),
        );
        m.insert(
            "has_typed_api".into(),
            json!(matches!(
                r.api_layer,
                crate::recipe::ApiLayer::Ninja | crate::recipe::ApiLayer::Drf
            )),
        );
        m.insert(
            "frontend_variant".into(),
            json!(r.frontend_variant.as_str()),
        );
        m.insert(
            "is_full_template".into(),
            json!(matches!(r.frontend_variant, crate::recipe::FrontendVariant::Full)),
        );
        m.insert(
            "is_skeleton".into(),
            json!(matches!(r.frontend_variant, crate::recipe::FrontendVariant::Skeleton)),
        );
        m
    }

    /// Engine-side extras the templates can address under `bakery.*` —
    /// random secrets, the current year, the bakery binary version. Booleans
    /// re-derived from the recipe live in [`recipe_fields`]; we don't repeat
    /// them here.
    fn engine_extras(r: &Recipe) -> Map<String, serde_json::Value> {
        let mut m = Map::new();
        m.insert("year".into(), json!(current_year()));
        m.insert("django_secret_key".into(), json!(secret_key(50)));

        // DB password — one value reused by the DB service AND `DATABASE_URL`, so the
        // auth flow stays in sync. `postgres_password` is kept as a back-compat alias.
        let db_password = json!(resolve_or_gen(&r.db_password, 40));
        m.insert("db_password".into(), db_password.clone());
        m.insert("postgres_password".into(), db_password);

        m.insert("redis_password".into(), json!(resolve_or_gen(&r.redis_password, 32)));
        m.insert("traefik_basic_auth".into(), json!(secret_key(32)));

        // Initial Django superuser, seeded idempotently on first boot. Email falls back to
        // the author's; password is generated unless the user typed one at setup.
        let superuser_email = if r.superuser_email.trim().is_empty() {
            r.author_email.clone()
        } else {
            r.superuser_email.clone()
        };
        m.insert("superuser_email".into(), json!(superuser_email));
        m.insert(
            "superuser_password".into(),
            json!(resolve_or_gen(&r.superuser_password, 32)),
        );

        // Celery Flower dashboard credentials (only consumed when Celery is enabled).
        let flower_user = if r.flower_user.trim().is_empty() {
            "flower".to_string()
        } else {
            r.flower_user.clone()
        };
        m.insert("flower_user".into(), json!(flower_user));
        m.insert(
            "flower_password".into(),
            json!(resolve_or_gen(&r.flower_password, 24)),
        );

        m.insert("allowed_hosts".into(), json!(r.allowed_hosts));

        // Non-secret but unguessable: admin URL suffix. Defends against `/admin/` scanners
        // and reduces the noise of automated brute-force on the auth endpoint.
        let admin_url_suffix = if r.admin_url_suffix.trim().is_empty() {
            secret_key(16).to_lowercase()
        } else {
            r.admin_url_suffix.clone()
        };
        m.insert("admin_url_suffix".into(), json!(admin_url_suffix));

        m.insert("bakery_version".into(), json!(env!("CARGO_PKG_VERSION")));
        m
    }
}

/// Use the user's explicit value if they provided one at setup, otherwise generate a fresh
/// strong secret. Backs the "generate a default, let the user accept-or-override" UX.
fn resolve_or_gen(value: &str, len: usize) -> String {
    if value.trim().is_empty() {
        secret_key(len)
    } else {
        value.to_string()
    }
}

fn current_year() -> i32 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    // Approximate calendar conversion; only need the year. Days since epoch ÷ 365.2425 + 1970.
    let days = secs / 86_400;
    1970 + (days as f64 / 365.2425) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::{
        ApiLayer, ContainerSetup, Frontend, RelationalDb,
    };

    #[test]
    fn recipe_fields_exposes_computed_booleans() {
        let r = Recipe::defaults();
        let block = Context::recipe_fields(&r);
        assert_eq!(block.get("is_postgres"), Some(&serde_json::json!(true)));
        assert_eq!(block.get("is_sqlite"), Some(&serde_json::json!(false)));
        assert_eq!(block.get("has_api"), Some(&serde_json::json!(true)));
        assert_eq!(block.get("wants_docker"), Some(&serde_json::json!(true)));
        assert_eq!(block.get("wants_traefik"), Some(&serde_json::json!(true)));
        assert_eq!(block.get("has_frontend_spa"), Some(&serde_json::json!(false)));
    }

    #[test]
    fn recipe_fields_flips_booleans_on_minimal_recipe() {
        let mut r = Recipe::defaults();
        r.relational_db = RelationalDb::Sqlite;
        r.api_layer = ApiLayer::None;
        r.frontend = Frontend::None;
        r.container_setup = ContainerSetup::None;
        let block = Context::recipe_fields(&r);
        assert_eq!(block.get("is_postgres"), Some(&serde_json::json!(false)));
        assert_eq!(block.get("is_sqlite"), Some(&serde_json::json!(true)));
        assert_eq!(block.get("has_api"), Some(&serde_json::json!(false)));
        assert_eq!(block.get("wants_docker"), Some(&serde_json::json!(false)));
        assert_eq!(block.get("wants_traefik"), Some(&serde_json::json!(false)));
    }

    #[test]
    fn engine_extras_include_secret_key_and_year() {
        let r = Recipe::defaults();
        let block = Context::engine_extras(&r);
        let key = block.get("django_secret_key").and_then(|v| v.as_str()).unwrap();
        assert_eq!(key.len(), 50);
        let pg = block.get("postgres_password").and_then(|v| v.as_str()).unwrap();
        assert_eq!(pg.len(), 40);
        let year = block.get("year").and_then(|v| v.as_i64()).unwrap();
        assert!(year >= 2024 && year <= 2100, "year sanity check: {year}");
    }

    #[test]
    fn recipe_fields_exposes_slug_variants() {
        let mut r = Recipe::defaults();
        r.project_slug = "my_cool_app".into();
        let block = Context::recipe_fields(&r);
        assert_eq!(
            block.get("project_module").and_then(|v| v.as_str()),
            Some("my_cool_app")
        );
        assert_eq!(
            block.get("project_kebab").and_then(|v| v.as_str()),
            Some("my-cool-app")
        );
        assert_eq!(
            block.get("project_camel").and_then(|v| v.as_str()),
            Some("MyCoolApp")
        );
        assert_eq!(
            block.get("project_shouty").and_then(|v| v.as_str()),
            Some("MY_COOL_APP")
        );
    }

    #[test]
    fn secret_key_alphabet_is_url_safe() {
        // URL-safe alphabet only — no shell-special chars that would corrupt .env parsing
        // or shell interpolation. Audit fix for the secret-key alphabet finding.
        let key = secret_key(128);
        assert_eq!(key.len(), 128);
        for c in key.chars() {
            assert!(
                c.is_ascii_alphanumeric() || c == '-' || c == '_',
                "secret_key contains unsafe character {c:?} in {key:?}"
            );
        }
    }

    #[test]
    fn secret_key_has_no_shell_special_chars() {
        // Belt and braces: this set must NEVER appear regardless of length.
        let key = secret_key(256);
        for forbidden in ['!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '"', '\'', '`', '\\'] {
            assert!(
                !key.contains(forbidden),
                "secret_key {key:?} contains forbidden char {forbidden:?}"
            );
        }
    }
}

/// Generate a Django-compatible secret key of `len` URL-safe characters.
///
/// Uses a ChaCha20 CSPRNG seeded from OS entropy (via the OS-seeded thread RNG —
/// rand 0.10 removed the direct `from_os_rng` constructor). Output is suitable for
/// `SECRET_KEY`, `POSTGRES_PASSWORD`, etc. — never logged, only written to `.env*` files
/// inside the generated project (which is `.gitignore`'d by default).
pub fn secret_key(len: usize) -> String {
    // URL-safe alphabet only (64 chars). At 50 chars that's ~300 bits of entropy — well
    // above Django's 128-bit minimum and the OWASP ASVS L2 threshold. Critically, no
    // characters with special meaning in shells or `.env` files (`$`, `!`, `(`, `*`, `&`,
    // `#`, quotes) — secrets written into env files must round-trip through any parser.
    const ALPHABET: &[u8] =
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";
    let mut rng = rand_chacha::ChaCha20Rng::from_rng(&mut rand::rng());
    let mut out = String::with_capacity(len);
    for _ in 0..len {
        let b = ALPHABET.choose(&mut rng).copied().unwrap_or(b'x');
        out.push(b as char);
    }
    out
}
