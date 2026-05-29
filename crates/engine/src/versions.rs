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
            " (+https://github.com/Axionore/Django-Bakery)"
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
        pypi.extend(["structlog", "django-structlog", "opentelemetry-sdk"]);
    }
    match r.api_layer {
        crate::recipe::ApiLayer::Ninja => pypi.push("django-ninja"),
        crate::recipe::ApiLayer::Drf => {
            pypi.extend(["djangorestframework", "drf-spectacular"]);
        }
        crate::recipe::ApiLayer::GraphqlStrawberry => {
            pypi.extend(["strawberry-graphql", "strawberry-graphql-django"]);
        }
        crate::recipe::ApiLayer::GraphqlGraphene => pypi.push("graphene-django"),
        crate::recipe::ApiLayer::None => {}
    }
    if r.is_postgres() {
        pypi.push("psycopg");
    }
    if r.multi_tenant {
        pypi.push("django-tenants");
    }

    let mut npm = Vec::<&'static str>::new();
    match r.frontend {
        crate::recipe::Frontend::React => {
            npm.extend([
                "react",
                "react-dom",
                "vite",
                "typescript",
                "@vitejs/plugin-react",
            ]);
            if matches!(r.radix_flavor, Some(crate::recipe::RadixFlavor::Themes)) {
                npm.extend([
                    "@radix-ui/themes",
                    "@radix-ui/react-icons",
                    "@radix-ui/colors",
                ]);
            } else if matches!(r.radix_flavor, Some(crate::recipe::RadixFlavor::Primitives)) {
                npm.extend(["@radix-ui/react-dialog", "tailwindcss", "@tailwindcss/vite"]);
            }
        }
        crate::recipe::Frontend::Nuxt => {
            // `@nuxtjs/tailwindcss` was dropped — it's pinned to Tailwind v3
            // and breaks v4 builds; templates use `@tailwindcss/vite` instead.
            npm.extend(["nuxt", "vue", "typescript", "@tailwindcss/vite"]);
        }
        crate::recipe::Frontend::Vue => {
            npm.extend([
                "vue",
                "vue-router",
                "pinia",
                "@vitejs/plugin-vue",
                "vite",
                "typescript",
            ]);
        }
        crate::recipe::Frontend::Next => {
            npm.extend([
                "next",
                "react",
                "react-dom",
                "eslint-config-next",
                "typescript",
            ]);
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
    // PEP 440 prerelease segment is shaped `<N>[.<sep>?]<marker>[<N>]`, e.g.
    // `1.0a1`, `2.5b2`, `3.1rc1`, `4.0.dev0`. A naive `contains("a")` matches
    // any version string with the letter 'a' in it — e.g. `1.0.0+abi3` (local
    // version with an ABI tag) or `1.0.0.post1+ubuntu.alpha` — and quietly
    // rejects legitimate stable releases. Anchor each marker to a preceding
    // digit + (digit | end | '.') to match PEP 440's actual shape.
    let v = version.to_ascii_lowercase();
    let bytes = v.as_bytes();
    for marker in ["a", "b", "c", "rc", "alpha", "beta", "dev", "pre"] {
        let mut start = 0;
        while let Some(rel) = v[start..].find(marker) {
            let pos = start + rel;
            let prev = pos.checked_sub(1).and_then(|i| bytes.get(i)).copied();
            let after = pos + marker.len();
            let next = bytes.get(after).copied();
            let prev_is_digit = matches!(prev, Some(c) if c.is_ascii_digit());
            let next_ok =
                next.is_none() || matches!(next, Some(c) if c.is_ascii_digit() || c == b'.');
            if prev_is_digit && next_ok {
                return true;
            }
            start = pos + marker.len();
        }
    }
    // SemVer prerelease: `1.0.0-rc.1`, `1.0.0-alpha.2`. Require a `<digit>-<alnum>`
    // shape so we don't false-positive on stray hyphens (e.g. package names).
    if let Some(hyphen) = v.find('-') {
        let before = hyphen.checked_sub(1).and_then(|i| bytes.get(i)).copied();
        let after = bytes.get(hyphen + 1).copied();
        if matches!(before, Some(c) if c.is_ascii_digit())
            && matches!(after, Some(c) if c.is_ascii_alphanumeric())
        {
            return true;
        }
    }
    false
}

/// Compatibility checks — known incompatible combinations.
pub fn compat_check(recipe: &Recipe, versions: &VersionMap) -> Vec<String> {
    let mut warnings = Vec::new();
    if let Some(django) = versions.get("py.django") {
        if let Ok(v) = Version::parse(&semver_safe(django))
            && v.major < 6
        {
            warnings.push(format!(
                "Django {django} is below the 6.x baseline django-bakery targets; \
                 some templates may use 6.x-only APIs"
            ));
        }
        // django-stubs major must match Django major
        if let Some(stubs) = versions.get("py.django-stubs")
            && !majors_match(django, stubs)
        {
            warnings.push(format!(
                "django-stubs {stubs} may not match Django {django}; check the django-stubs release notes"
            ));
        }
    }
    if recipe.use_celery
        && let Some(c) = versions.get("py.celery")
        && let Ok(v) = Version::parse(&semver_safe(c))
        && (v.major < 5 || (v.major == 5 && v.minor < 4))
    {
        warnings.push(format!(
            "Celery {c} is below 5.4; Django 6 async ORM compatibility requires 5.4+"
        ));
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

#[cfg(test)]
// `defaults()` is a large bundled-data fn intentionally kept at the bottom of the file;
// relocating it purely to satisfy this stylistic lint would add churn with no benefit.
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_well_formed() {
        // Every value in the bundled defaults must look like `X.Y.Z` (or
        // `X.Y` for the few packages that only ship two segments). No empty
        // strings, no leading `^`/`~`, no obvious nonsense.
        for (k, v) in defaults() {
            assert!(!v.is_empty(), "{k} has empty value");
            assert!(
                !v.starts_with('^'),
                "{k} = {v} must not include a semver range marker"
            );
            assert!(
                !v.starts_with('~'),
                "{k} = {v} must not include a semver range marker"
            );
            assert!(
                v.chars().next().is_some_and(|c| c.is_ascii_digit()),
                "{k} = {v} must start with a digit"
            );
        }
    }

    #[test]
    fn defaults_have_no_known_phantom_versions() {
        // Regression test for the 2026-05-27 audit: these specific versions
        // were in the previous defaults snapshot but **did not exist on
        // upstream registries**. Re-checking against npm / PyPI today they
        // still don't exist as stable releases. If one re-appears here, the
        // resolver is producing broken `package.json` / `pyproject.toml` and
        // `pnpm install` / `uv sync` will fail on the generated project.
        let phantoms: &[(&str, &str)] = &[
            ("npm.vue", "3.6.0"),
            ("npm.@nuxtjs/tailwindcss", "7.0.0"),
            ("npm.@radix-ui/themes", "4.0.0"),
            ("npm.@radix-ui/colors", "4.0.0"),
            ("npm.vitest", "8.0.0"),
            ("npm.@vitest/coverage-v8", "8.0.0"),
            ("npm.@vitejs/plugin-react", "5.0.0"),
        ];
        let d = defaults();
        for (key, bad) in phantoms {
            if let Some(actual) = d.get(*key) {
                assert_ne!(
                    actual, bad,
                    "{key} re-acquired the phantom version {bad} — \
                     check `curl https://registry.npmjs.org/<pkg>/latest`"
                );
            }
        }
    }

    #[test]
    fn defaults_cover_every_package_requested_by_relevant_packages() {
        // Whatever `relevant_packages()` asks the resolver to fetch must
        // have a matching default — otherwise an offline run renders a
        // hole into the template. The previous version of this test only
        // exercised `Recipe::defaults()` (which is htmx-alpine + no celery
        // + ninja + no multi-tenant), so every branch of `relevant_packages`
        // that lit up only for OTHER toggles (multi_tenant, GraphqlStrawberry,
        // Nuxt, Vue, etc.) was tested vacuously and could lose its default
        // entry without breaking the suite. Walk a small matrix that turns
        // each toggle on at least once.
        use crate::recipe::{
            ApiLayer, CssFramework, Frontend, FrontendVariant, RadixFlavor, RelationalDb,
        };
        let d = defaults();
        let assert_covered = |r: &Recipe, label: &str| {
            let wanted = relevant_packages(r);
            for pkg in wanted.pypi {
                assert!(
                    d.contains_key(&format!("py.{pkg}")),
                    "defaults missing py.{pkg} (matrix variant: {label})"
                );
            }
            for pkg in wanted.npm {
                assert!(
                    d.contains_key(&format!("npm.{pkg}")),
                    "defaults missing npm.{pkg} (matrix variant: {label})"
                );
            }
        };
        assert_covered(&Recipe::defaults(), "defaults");
        for api in [
            ApiLayer::Ninja,
            ApiLayer::Drf,
            ApiLayer::GraphqlStrawberry,
            ApiLayer::GraphqlGraphene,
            ApiLayer::None,
        ] {
            let mut r = Recipe::defaults();
            r.api_layer = api;
            assert_covered(&r, &format!("api_layer={}", api.as_str()));
        }
        for (frontend, radix) in [
            (Frontend::HtmxAlpine, None),
            (Frontend::React, Some(RadixFlavor::Themes)),
            (Frontend::React, Some(RadixFlavor::Primitives)),
            (Frontend::Nuxt, None),
            (Frontend::Vue, None),
            (Frontend::Next, None),
        ] {
            let mut r = Recipe::defaults();
            r.frontend = frontend;
            r.frontend_variant = FrontendVariant::Full;
            r.radix_flavor = radix;
            assert_covered(&r, &format!("frontend={}", frontend.as_str()));
        }
        for css in [
            CssFramework::Tailwind,
            CssFramework::Bootstrap,
            CssFramework::None,
        ] {
            let mut r = Recipe::defaults();
            r.css_framework = css;
            assert_covered(&r, &format!("css={}", css.as_str()));
        }
        {
            let mut r = Recipe::defaults();
            r.multi_tenant = true;
            r.relational_db = RelationalDb::Postgres;
            assert_covered(&r, "multi_tenant=true");
        }
        {
            let mut r = Recipe::defaults();
            r.use_celery = true;
            assert_covered(&r, "use_celery=true");
        }
        {
            let mut r = Recipe::defaults();
            r.use_observability = true;
            assert_covered(&r, "use_observability=true");
        }
        {
            let mut r = Recipe::defaults();
            r.use_sentry = true;
            assert_covered(&r, "use_sentry=true");
        }
    }
}

/// Bundled defaults snapshot — actual `latest` on PyPI / npm registry as of
/// 2026-05-27, re-verified against the live registries. Used in `Offline`
/// mode and as the fallback when the network call in `Online` mode fails.
///
/// **Invariant**: every value here MUST exist on the upstream registry. The
/// earlier (2026-05-26) snapshot drifted on several packages — `vue@3.6`
/// didn't exist (latest 3.5.35; 3.6 was beta-only), `@nuxtjs/tailwindcss@7`
/// didn't exist (latest 6.14), `@radix-ui/themes@4` didn't exist (latest 3.3),
/// `vitest@8` didn't exist (v8 is the coverage engine plug, not vitest) —
/// which caused `cargo install django-bakery && django-bakery new --offline`
/// to produce projects whose `package.json` resolved to non-existent versions
/// and broke `pnpm install`. Every value here is now `curl
/// https://registry.npmjs.org/<pkg>/latest` / `https://pypi.org/pypi/<pkg>/json`
/// verified.
fn defaults() -> VersionMap {
    let mut m = VersionMap::new();
    for (k, v) in [
        // --- Python ecosystem ---
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
        ("py.django-celery-results", "2.6.0"),
        ("py.flower", "2.0.1"),
        ("py.sentry-sdk", "2.20.0"),
        ("py.structlog", "25.1.0"),
        ("py.django-structlog", "9.0.0"),
        ("py.opentelemetry-sdk", "1.30.0"),
        ("py.django-ninja", "1.4.0"),
        ("py.djangorestframework", "3.16.0"),
        ("py.drf-spectacular", "0.28.0"),
        ("py.strawberry-graphql", "0.316.0"),
        ("py.strawberry-graphql-django", "0.86.0"),
        ("py.graphene-django", "3.2.3"),
        ("py.psycopg", "3.2.4"),
        ("py.pymysql", "1.1.0"),
        ("py.pydantic", "2.10.0"),
        ("py.django-tenants", "3.10.1"),
        // --- Frontend ecosystem ---
        ("npm.react", "19.2.5"),
        ("npm.react-dom", "19.2.5"),
        ("npm.react-router", "7.15.0"),
        ("npm.vite", "8.0.14"),
        ("npm.typescript", "6.0.3"),
        ("npm.@vitejs/plugin-react", "6.0.2"),
        ("npm.@vitejs/plugin-vue", "6.0.7"),
        ("npm.@radix-ui/themes", "3.3.0"),
        ("npm.@radix-ui/react-icons", "1.3.2"),
        ("npm.@radix-ui/colors", "3.0.0"),
        ("npm.@radix-ui/react-dialog", "1.1.2"),
        ("npm.tailwindcss", "4.3.0"),
        ("npm.@tailwindcss/vite", "4.3.0"),
        ("npm.@tailwindcss/cli", "4.3.0"),
        ("npm.nuxt", "4.4.6"),
        ("npm.vue", "3.5.35"),
        ("npm.vue-router", "4.5.0"),
        ("npm.pinia", "3.0.4"),
        ("npm.@pinia/nuxt", "0.11.3"),
        ("npm.@vueuse/core", "14.3.0"),
        ("npm.@vueuse/nuxt", "14.3.0"),
        ("npm.@vue/test-utils", "2.4.10"),
        ("npm.next", "16.2.6"),
        ("npm.eslint-config-next", "16.2.6"),
        ("npm.htmx.org", "2.0.4"),
        ("npm.alpinejs", "3.14.0"),
        ("npm.bootstrap", "5.3.6"),
        ("npm.vitest", "4.1.7"),
        ("npm.@vitest/coverage-v8", "4.1.7"),
        ("npm.jsdom", "29.1.1"),
        ("npm.happy-dom", "20.9.0"),
        ("npm.eslint", "10.4.0"),
        ("npm.@nuxt/eslint", "1.15.2"),
        ("npm.@nuxt/test-utils", "4.0.3"),
        ("npm.zod", "4.4.3"),
        ("npm.zustand", "5.0.13"),
        ("npm.@tanstack/react-query", "5.100.14"),
        ("npm.@playwright/test", "1.60.0"),
        ("npm.openapi-typescript", "7.13.0"),
        ("npm.prettier", "3.8.3"),
    ] {
        m.insert(k.into(), v.into());
    }
    m
}
