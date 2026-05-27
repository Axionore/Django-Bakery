//! Rendering engine for django-bakery.
//!
//! Given a [`Recipe`], walks the embedded template tree, renders paths + contents through
//! a Jinja2-compatible engine (minijinja), honors `__SKIP__` sentinels to omit
//! conditional subtrees, and writes the result to an output directory.

pub mod context;
pub mod error;
pub mod filters;
pub mod post_gen;
pub mod recipe;
pub mod render;
pub mod versions;

pub use context::Context;
pub use error::{Error, Result};
pub use recipe::{
    ApiLayer, CeleryBroker, CiProvider, ContainerSetup, CssFramework, DjangoVersion, Frontend,
    GraphDb, JsLanguage, License, Mode, ProdEmail, PythonVersion, RadixFlavor, Recipe,
    RelationalDb, Storage, TypeChecker, VersionControl,
};
pub use render::{RenderOptions, RenderReport, render};
pub use versions::{ResolveMode, VersionMap, compat_check, resolve as resolve_versions};
