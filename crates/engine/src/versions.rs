//! Version resolver — at generation time, query PyPI and the npm registry for the latest
//! stable version of each pinned dependency, and inject the results into the render
//! context under `bakery.versions.<name>`.
//!
//! Defaults are baked in so offline generation still works. The CLI's `--offline` flag
//! short-circuits the network calls.

use std::collections::BTreeMap;
use std::time::Duration;

use semver::Version;
use serde_json::Value;
use tracing::{debug, warn};

use crate::recipe::Recipe;

/// Resolved version table, keyed by canonical package name.
pub type VersionMap = BTreeMap<String, String>;

#[derive(Debug, Clone, Copy)]
pub enum ResolveMode {
    /// Hit PyPI / npm to fetch latest stable; fall back to a bundled default on failure.
    Online,
    /// Use bundled defaults only.
    Offline,
}

/// Resolve every version we care about for the given recipe.
pub fn resolve(recipe: &Recipe, mode: ResolveMode) -> VersionMap {
    let mut out = defaults();
    let needed = relevant_packages(recipe);
    if !matches!(mode, ResolveMode::Online) {
        return out;
    }

    let agent = build_agent();
    for pkg in &needed.pypi {
        match latest_pypi(&agent, pkg) {
            Ok(v) => {
                debug!(%pkg, %v, "resolved");
                out.insert(format!("py.{pkg}"), v);
            }
            Err(err) => warn!(%pkg, %err, "pypi lookup failed; using default"),
        }
    }
    for pkg in &needed.npm {
        match latest_npm(&agent, pkg) {
            Ok(v) => {
                debug!(%pkg, %v, "resolved (npm)");
                out.insert(format!("npm.{pkg}"), v);
            }
            Err(err) => warn!(%pkg, %err, "npm lookup failed; using default"),
        }
    }
    out
}

fn build_agent() -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .user_agent(concat!(
            "django-bakery/",
            env!("CARGO_PKG_VERSION"),
            " (+https://github.com/charlesasobel/django-bakery)"
        ))
        .build();
    ureq::Agent::new_with_config(config)
}

struct Wanted {
    pypi: Vec<&'static str>,
    npm: Vec<&'static str>,
}

fn relevant_packages(r: &Recipe) -> Wanted {
    let mut pypi = vec![
        "django",
        "django-environ",
        "django-allauth",
        "django-csp",
        "django-cors-headers",
        "django-ratelimit",
        "argon2-cffi",
        "gunicorn",
        "uvicorn",
        "whitenoise",
        "pytest",
        "pytest-django",
        "ruff",
        "factory-boy",
    ];
    if r.use_celery {
        pypi.extend(["celery", "django-celery-beat", "flower"]);
    }
    if r.use_sentry {
        pypi.push("sentry-sdk");
    }
    if r.use_observability {
        pypi.extend([
            "structlog",
            "django-structlog",
            "opentelemetry-sdk",
        ]);
    }
    match r.api_layer {
        crate::recipe::ApiLayer::Ninja => pypi.push("django-ninja"),
        crate::recipe::ApiLayer::Drf => {
            pypi.extend(["djangorestframework", "drf-spectacular"]);
        }
        crate::recipe::ApiLayer::GraphqlStrawberry => pypi.push("strawberry-graphql"),
        crate::recipe::ApiLayer::GraphqlGraphene => pypi.push("graphene-django"),
        crate::recipe::ApiLayer::None => {}
    }
    if r.is_postgres() {
        pypi.push("psycopg");
    }

    let mut npm = Vec::<&'static str>::new();
    match r.frontend {
        crate::recipe::Frontend::React => {
            npm.extend(["react", "react-dom", "vite", "typescript", "@vitejs/plugin-react"]);
            if matches!(r.radix_flavor, Some(crate::recipe::RadixFlavor::Themes)) {
                npm.extend(["@radix-ui/themes", "@radix-ui/react-icons", "@radix-ui/colors"]);
            } else if matches!(r.radix_flavor, Some(crate::recipe::RadixFlavor::Primitives)) {
                npm.extend(["@radix-ui/react-dialog", "tailwindcss", "@tailwindcss/vite"]);
            }
        }
        crate::recipe::Frontend::Nuxt => {
            npm.extend(["nuxt", "vue", "typescript", "@nuxtjs/tailwindcss"]);
        }
        crate::recipe::Frontend::HtmxAlpine => {
            npm.extend(["htmx.org", "alpinejs"]);
        }
        _ => {}
    }
    if matches!(r.css_framework, crate::recipe::CssFramework::Tailwind) {
        npm.push("tailwindcss");
        npm.push("@tailwindcss/cli");
    }
    if matches!(r.css_framework, crate::recipe::CssFramework::Bootstrap) {
        npm.push("bootstrap");
    }
    Wanted { pypi, npm }
}

fn latest_pypi(agent: &ureq::Agent, pkg: &str) -> Result<String, String> {
    let url = format!("https://pypi.org/pypi/{pkg}/json");
    let body: Value = agent
        .get(&url)
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_json()
        .map_err(|e| e.to_string())?;
    let info = body.get("info").ok_or("no info")?;
    let version = info
        .get("version")
        .and_then(Value::as_str)
        .ok_or("no version")?;
    if is_prerelease(version) {
        return Err(format!("latest is prerelease: {version}"));
    }
    Ok(version.to_string())
}

fn latest_npm(agent: &ureq::Agent, pkg: &str) -> Result<String, String> {
    let url = format!("https://registry.npmjs.org/{pkg}/latest");
    let body: Value = agent
        .get(&url)
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_json()
        .map_err(|e| e.to_string())?;
    body.get("version")
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .ok_or("no version".into())
}

fn is_prerelease(version: &str) -> bool {
    // PEP 440 prerelease markers
    let v = version.to_ascii_lowercase();
    v.contains("a") || v.contains("b") || v.contains("rc") || v.contains("dev") || v.contains("pre")
        // SemVer prerelease
        || v.contains('-')
}

/// Compatibility checks — known incompatible combinations.
pub fn compat_check(recipe: &Recipe, versions: &VersionMap) -> Vec<String> {
    let mut warnings = Vec::new();
    if let Some(django) = versions.get("py.django") {
        if let Ok(v) = Version::parse(&semver_safe(django)) {
            if v.major < 6 {
                warnings.push(format!(
                    "Django {django} is below the 6.x baseline django-bakery targets; \
                     some templates may use 6.x-only APIs"
                ));
            }
        }
        // django-stubs major must match Django major
        if let Some(stubs) = versions.get("py.django-stubs") {
            if !majors_match(django, stubs) {
                warnings.push(format!(
                    "django-stubs {stubs} may not match Django {django}; check the django-stubs release notes"
                ));
            }
        }
    }
    if recipe.use_celery {
        if let Some(c) = versions.get("py.celery") {
            if let Ok(v) = Version::parse(&semver_safe(c)) {
                if v.major < 5 || (v.major == 5 && v.minor < 4) {
                    warnings.push(format!(
                        "Celery {c} is below 5.4; Django 6 async ORM compatibility requires 5.4+"
                    ));
                }
            }
        }
    }
    warnings
}

fn semver_safe(v: &str) -> String {
    // PEP 440 → ish semver: keep major.minor.patch, drop trailing markers
    let mut out = String::new();
    let mut dots = 0;
    for c in v.chars() {
        if c.is_ascii_digit() {
            out.push(c);
        } else if c == '.' && dots < 2 {
            out.push(c);
            dots += 1;
        } else {
            break;
        }
    }
    while dots < 2 {
        out.push_str(".0");
        dots += 1;
    }
    out
}

fn majors_match(a: &str, b: &str) -> bool {
    let am = a.split('.').next().unwrap_or("");
    let bm = b.split('.').next().unwrap_or("");
    am == bm
}

/// Bundled defaults snapshot — "latest stable" as of 2026-05-26. Used offline or as a
/// fallback when the registry call fails.
fn defaults() -> VersionMap {
    let mut m = VersionMap::new();
    // Python ecosystem
    for (k, v) in [
        ("py.django", "6.0.0"),
        ("py.django-environ", "0.12.0"),
        ("py.django-allauth", "65.4.0"),
        ("py.django-csp", "4.0"),
        ("py.django-cors-headers", "4.6.0"),
        ("py.django-ratelimit", "4.1.0"),
        ("py.django-extensions", "4.0"),
        ("py.django-stubs", "6.0.0"),
        ("py.argon2-cffi", "23.1.0"),
        ("py.gunicorn", "23.0.0"),
        ("py.uvicorn", "0.35.0"),
        ("py.whitenoise", "6.8.2"),
        ("py.pytest", "8.3.4"),
        ("py.pytest-django", "4.9.0"),
        ("py.ruff", "0.9.0"),
        ("py.factory-boy", "3.3.1"),
        ("py.celery", "5.5.0"),
        ("py.django-celery-beat", "2.7.0"),
        ("py.flower", "2.0.1"),
        ("py.sentry-sdk", "2.20.0"),
        ("py.structlog", "25.1.0"),
        ("py.django-structlog", "9.0.0"),
        ("py.opentelemetry-sdk", "1.30.0"),
        ("py.django-ninja", "1.4.0"),
        ("py.djangorestframework", "3.16.0"),
        ("py.drf-spectacular", "0.28.0"),
        ("py.strawberry-graphql", "0.270.0"),
        ("py.graphene-django", "3.2.3"),
        ("py.psycopg", "3.2.4"),
        ("py.pydantic", "2.10.0"),
        // Frontend ecosystem
        ("npm.react", "19.2.0"),
        ("npm.react-dom", "19.2.0"),
        ("npm.vite", "7.0.0"),
        ("npm.typescript", "6.0.0"),
        ("npm.@vitejs/plugin-react", "5.0.0"),
        ("npm.@radix-ui/themes", "4.0.0"),
        ("npm.@radix-ui/react-icons", "2.0.0"),
        ("npm.@radix-ui/colors", "4.0.0"),
        ("npm.@radix-ui/react-dialog", "2.0.0"),
        ("npm.tailwindcss", "4.1.0"),
        ("npm.@tailwindcss/vite", "4.1.0"),
        ("npm.@tailwindcss/cli", "4.1.0"),
        ("npm.nuxt", "4.1.0"),
        ("npm.vue", "3.6.0"),
        ("npm.@nuxtjs/tailwindcss", "7.0.0"),
        ("npm.htmx.org", "2.0.4"),
        ("npm.alpinejs", "3.14.0"),
        ("npm.bootstrap", "5.3.6"),
        ("npm.vitest", "8.0.0"),
        ("npm.@playwright/test", "1.50.0"),
    ] {
        m.insert(k.into(), v.into());
    }
    m
}
