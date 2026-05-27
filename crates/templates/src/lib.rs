//! Embedded template assets for django-bakery.
//!
//! The project template tree under `crates/templates/files/` is compiled into the binary
//! via [`include_dir`]. The engine crate consumes it via [`PROJECT_TEMPLATE`].

use include_dir::{Dir, include_dir};

/// The root of the templated Django project, rooted at the templated project slug directory.
pub static PROJECT_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/files");

/// The directory inside `PROJECT_TEMPLATE` whose name (after Jinja rendering) becomes the
/// generated project's root folder.
pub const PROJECT_ROOT_TOKEN: &str = "{{cookiecutter.project_slug}}";
