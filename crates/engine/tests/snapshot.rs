//! Snapshot tests — pin the *tree shape* of canonical recipes via insta.
//!
//! Run as: `cargo test -p django-bakery-engine --test snapshot`
//!
//! When intentional template changes shift the tree, accept the new snapshots
//! with `cargo insta review` (or `cargo insta accept` to bless everything
//! without inspection — only do that when you've already manually diff'd).
//!
//! Why a separate test surface: the content-level `render.rs` tests assert
//! specific sentinels in specific files (sharp + low-recall). Snapshots
//! catch the long-tail regression — "a file accidentally appeared or
//! disappeared because of a misnamed `__SKIP__` conditional or stray
//! `frontend != 'react'` typo" — without needing the test author to
//! anticipate every possible drift.

use std::path::Path;

use django_bakery_engine::{
    ApiLayer, CiProvider, ContainerSetup, CssFramework, Frontend, FrontendVariant, GraphDb,
    JsLanguage, ProdEmail, RadixFlavor, Recipe, RelationalDb, RenderOptions, ResolveMode, Storage,
    TypeChecker, render,
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

/// Walk `root` and return a deterministic sorted list of relative paths.
/// Snapshot this — the diff is human-readable and catches accidental
/// adds/removes across the whole tree.
fn tree_listing(root: &Path) -> String {
    let mut entries: Vec<String> = walkdir::WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            e.path()
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    entries.sort();
    entries.join("\n")
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

fn react_full_recipe() -> Recipe {
    let mut r = Recipe::defaults();
    r.project_slug = "react_app".into();
    r.frontend = Frontend::React;
    r.frontend_variant = FrontendVariant::Full;
    r.radix_flavor = Some(RadixFlavor::Themes);
    r.js_language = JsLanguage::Typescript;
    r.js_testing = true;
    r
}

fn nuxt_full_recipe() -> Recipe {
    let mut r = Recipe::defaults();
    r.project_slug = "nuxt_app".into();
    r.frontend = Frontend::Nuxt;
    r.frontend_variant = FrontendVariant::Full;
    r.js_language = JsLanguage::Typescript;
    r.js_testing = true;
    r
}

#[test]
fn snapshot_tree_minimal_recipe() {
    let (_tmp, root) = render_recipe(&minimal_recipe());
    insta::assert_snapshot!(tree_listing(&root));
}

#[test]
fn snapshot_tree_defaults_recipe() {
    // Recipe::defaults() — htmx + tailwind + ninja + postgres + celery + sentry
    // + mailpit + S3 + AWS storage + traefik. The "production" maximum.
    let (_tmp, root) = render_recipe(&Recipe::defaults());
    insta::assert_snapshot!(tree_listing(&root));
}

#[test]
fn snapshot_tree_react_full_recipe() {
    let (_tmp, root) = render_recipe(&react_full_recipe());
    insta::assert_snapshot!(tree_listing(&root));
}

#[test]
fn snapshot_tree_nuxt_full_recipe() {
    let (_tmp, root) = render_recipe(&nuxt_full_recipe());
    insta::assert_snapshot!(tree_listing(&root));
}
