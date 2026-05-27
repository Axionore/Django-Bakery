//! Builds the rendering context — a `minijinja::Value` — from a [`Recipe`].
//!
//! Templates address everything via `cookiecutter.*` for parity with cookiecutter-django
//! (so existing community templates port without rewriting), plus a few `bakery.*`
//! computed helpers we expose.

use std::time::{SystemTime, UNIX_EPOCH};

use heck::{ToKebabCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use minijinja::Value;
use rand::SeedableRng;
use rand::seq::IndexedRandom;
use serde_json::{Map, json};

use crate::recipe::Recipe;

/// Computed context exposed to templates under `cookiecutter` and `bakery`.
pub struct Context;

impl Context {
    /// Build a [`minijinja::Value`] tree from a recipe with default (bundled) versions.
    pub fn build(recipe: &Recipe) -> Value {
        let versions = crate::versions::resolve(recipe, crate::versions::ResolveMode::Offline);
        Self::build_with_versions(recipe, &versions)
    }

    /// Build a context with a pre-resolved version map (the normal path during render).
    pub fn build_with_versions(recipe: &Recipe, versions: &crate::versions::VersionMap) -> Value {
        let cookiecutter = Self::cookiecutter_block(recipe);
        let mut bakery = Self::bakery_block(recipe);

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

        let root = json!({
            "cookiecutter": cookiecutter,
            "bakery": bakery,
        });
        Value::from_serialize(&root)
    }

    fn cookiecutter_block(r: &Recipe) -> Map<String, serde_json::Value> {
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
        m.insert("type_checker".into(), json!(r.type_checker.as_str()));
        m.insert("use_pre_commit".into(), json!(r.use_pre_commit));
        m.insert("ci".into(), json!(r.ci.as_str()));
        m.insert("container_setup".into(), json!(r.container_setup.as_str()));
        m.insert("version_control".into(), json!(r.version_control.as_str()));

        // Computed booleans — referenced as `cookiecutter.is_postgres` etc. throughout
        // the templates. (Cookiecutter-django bakes derived fields into cookiecutter.json
        // pre/post hooks; we do it eagerly here.)
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
            "has_api".into(),
            json!(!matches!(r.api_layer, crate::recipe::ApiLayer::None)),
        );
        m.insert(
            "has_frontend_spa".into(),
            json!(matches!(
                r.frontend,
                crate::recipe::Frontend::React | crate::recipe::Frontend::Nuxt
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
        m
    }

    fn bakery_block(r: &Recipe) -> Map<String, serde_json::Value> {
        let mut m = Map::new();
        let year = current_year();
        m.insert("year".into(), json!(year));
        m.insert("django_secret_key".into(), json!(secret_key(50)));
        m.insert("postgres_password".into(), json!(secret_key(40)));
        m.insert("redis_password".into(), json!(secret_key(32)));
        m.insert("traefik_basic_auth".into(), json!(secret_key(32)));
        m.insert(
            "bakery_version".into(),
            json!(env!("CARGO_PKG_VERSION")),
        );
        // computed booleans templates love
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
            "has_api".into(),
            json!(!matches!(r.api_layer, crate::recipe::ApiLayer::None)),
        );
        m.insert(
            "has_frontend_spa".into(),
            json!(matches!(
                r.frontend,
                crate::recipe::Frontend::React | crate::recipe::Frontend::Nuxt
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
        m
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

/// Generate a Django-compatible secret key of `len` URL-safe characters.
///
/// Uses a ChaCha20-based CSPRNG seeded from OS entropy. Output is suitable for
/// `SECRET_KEY`, `POSTGRES_PASSWORD`, etc. — never logged, only written to `.env*` files
/// inside the generated project (which is `.gitignore`'d by default).
pub fn secret_key(len: usize) -> String {
    const ALPHABET: &[u8] =
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_!@#$%^&*()";
    let mut rng = rand_chacha::ChaCha20Rng::from_os_rng();
    let mut out = String::with_capacity(len);
    for _ in 0..len {
        let b = ALPHABET.choose(&mut rng).copied().unwrap_or(b'x');
        out.push(b as char);
    }
    out
}
