//! The render walker.
//!
//! Walks the embedded `PROJECT_TEMPLATE` virtual filesystem in deterministic order,
//! renders each path and file body through minijinja, honors `__SKIP__` sentinels for
//! conditional inclusion, and writes results to a target directory.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use include_dir::DirEntry;
use minijinja::{AutoEscape, Environment, UndefinedBehavior, Value};
use regex::Regex;
use tracing::{debug, trace};

use crate::error::{Error, Result};
use crate::filters;
use crate::recipe::Recipe;
use crate::{Context, post_gen};

const SKIP_SENTINEL: &str = "__SKIP__";

/// Caller-supplied options for one render call.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub output_dir: PathBuf,
    /// Overwrite an existing non-empty output directory.
    pub force: bool,
    /// Run post-generation hooks (chmod scripts, git init, etc.).
    pub run_hooks: bool,
    /// Initialize the recipe's VCS + first commit.
    pub git_init: bool,
    /// Run `uv sync` and friends inside the generated project.
    pub bootstrap: bool,
    /// How to resolve dependency versions.
    pub version_mode: crate::versions::ResolveMode,
    /// Treat compatibility-check warnings as hard errors.
    pub strict_compat: bool,
}

impl RenderOptions {
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
            force: false,
            run_hooks: true,
            git_init: true,
            bootstrap: false,
            version_mode: crate::versions::ResolveMode::Online,
            strict_compat: false,
        }
    }
}

/// Summary returned after a render.
#[derive(Debug, Clone)]
pub struct RenderReport {
    pub project_root: PathBuf,
    pub files_written: usize,
    pub directories_created: usize,
    pub elapsed_ms: u128,
    pub compat_warnings: Vec<String>,
}

/// Render a [`Recipe`] using the embedded project template tree.
pub fn render(recipe: &Recipe, options: &RenderOptions) -> Result<RenderReport> {
    recipe.validate().map_err(Error::InvalidRecipe)?;

    let started = Instant::now();
    let versions = crate::versions::resolve(recipe, options.version_mode);
    let compat_warnings = crate::versions::compat_check(recipe, &versions);
    if options.strict_compat && !compat_warnings.is_empty() {
        return Err(Error::InvalidRecipe(format!(
            "compatibility check failed:\n  - {}",
            compat_warnings.join("\n  - ")
        )));
    }
    let ctx = Context::build_with_versions(recipe, &versions);
    let mut env = make_env();
    filters::register(&mut env);

    let target_root = options.output_dir.join(&recipe.project_slug);
    prepare_target(&target_root, options.force)?;

    let template_root = &django_bakery_templates::PROJECT_TEMPLATE;
    let mut writer = RenderWriter::new(&env, &ctx, &target_root);

    // The embedded tree has exactly one top-level directory:
    // `{{bakery.project_slug}}/`. We don't want to emit *that* dir (it's the
    // generated project's root, already created as `target_root`); we walk its CHILDREN.
    for entry in template_root.entries() {
        if let DirEntry::Dir(d) = entry {
            for child in d.entries() {
                writer.process(child, Path::new(""))?;
            }
        }
        // (Stray top-level files in `files/` are ignored — there should be none.)
    }

    if options.run_hooks {
        post_gen::after_render(&target_root, recipe)?;
    }
    if options.git_init {
        post_gen::vcs_init(&target_root, recipe.version_control)?;
    }
    if options.bootstrap {
        post_gen::bootstrap(&target_root, recipe)?;
    }

    let files_written = writer.files_written;
    let dirs_created = writer.dirs_created;
    drop(writer);
    Ok(RenderReport {
        project_root: target_root,
        files_written,
        directories_created: dirs_created,
        elapsed_ms: started.elapsed().as_millis(),
        compat_warnings,
    })
}

fn make_env() -> Environment<'static> {
    let mut env = Environment::new();
    // We render Python, YAML, TOML, Dockerfile — NOT HTML by default. The few HTML
    // templates in the tree end in `.html` and we re-enable escaping for those only.
    env.set_auto_escape_callback(|name| {
        if name.ends_with(".html") || name.ends_with(".html.j2") {
            AutoEscape::Html
        } else {
            AutoEscape::None
        }
    });
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.set_keep_trailing_newline(true);
    env
}

fn prepare_target(target: &Path, force: bool) -> Result<()> {
    if target.exists() {
        let is_empty = fs::read_dir(target)
            .map_err(|source| Error::Io {
                path: Some(target.to_path_buf()),
                source,
            })?
            .next()
            .is_none();
        if !is_empty && !force {
            return Err(Error::OutputDirNotEmpty(target.to_path_buf()));
        }
        if force && !is_empty {
            fs::remove_dir_all(target).map_err(|source| Error::Io {
                path: Some(target.to_path_buf()),
                source,
            })?;
            fs::create_dir_all(target).map_err(|source| Error::Io {
                path: Some(target.to_path_buf()),
                source,
            })?;
        }
    } else {
        fs::create_dir_all(target).map_err(|source| Error::Io {
            path: Some(target.to_path_buf()),
            source,
        })?;
    }
    Ok(())
}

struct RenderWriter<'a> {
    env: &'a Environment<'static>,
    ctx: &'a Value,
    target_root: &'a Path,
    binary_globs: HashSet<&'static str>,
    files_written: usize,
    dirs_created: usize,
}

impl<'a> RenderWriter<'a> {
    fn new(env: &'a Environment<'static>, ctx: &'a Value, target_root: &'a Path) -> Self {
        // Globs treated as binary: copy bytes unchanged, don't render.
        let mut binary_globs = HashSet::new();
        binary_globs.insert("png");
        binary_globs.insert("jpg");
        binary_globs.insert("jpeg");
        binary_globs.insert("gif");
        binary_globs.insert("ico");
        binary_globs.insert("webp");
        binary_globs.insert("woff");
        binary_globs.insert("woff2");
        binary_globs.insert("ttf");
        binary_globs.insert("eot");
        binary_globs.insert("otf");
        binary_globs.insert("mp4");
        binary_globs.insert("pdf");
        Self {
            env,
            ctx,
            target_root,
            binary_globs,
            files_written: 0,
            dirs_created: 0,
        }
    }

    fn process(&mut self, entry: &DirEntry<'_>, rel_parent: &Path) -> Result<()> {
        match entry {
            DirEntry::Dir(d) => {
                let raw_name = d
                    .path()
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();
                let rendered = self.render_str(raw_name)?;
                if rendered.contains(SKIP_SENTINEL) || rendered.trim().is_empty() {
                    trace!(?raw_name, "skipping conditional dir");
                    return Ok(());
                }
                let rendered = strip_jinja_ext(&rendered);
                let rel = rel_parent.join(&rendered);
                let abs = self.target_root.join(&rel);
                guard_path_within_root(&abs, self.target_root)?;
                fs::create_dir_all(&abs).map_err(|source| Error::Io {
                    path: Some(abs.clone()),
                    source,
                })?;
                self.dirs_created += 1;
                for child in d.entries() {
                    self.process(child, &rel)?;
                }
                Ok(())
            }
            DirEntry::File(f) => {
                let raw_name = f
                    .path()
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();
                let rendered_name = self.render_str(raw_name)?;
                if rendered_name.contains(SKIP_SENTINEL) || rendered_name.trim().is_empty() {
                    trace!(?raw_name, "skipping conditional file");
                    return Ok(());
                }
                // A trailing `.j2` marker means: run the body through Jinja and strip the suffix.
                // Without `.j2`, the file is byte-copied verbatim — safe for JS / CSS / binary
                // assets where curly braces would otherwise collide with Jinja's syntax.
                let needs_render = rendered_name.ends_with(".j2");
                let final_name = strip_jinja_ext(&rendered_name);
                let rel = rel_parent.join(final_name);
                let abs = self.target_root.join(&rel);
                guard_path_within_root(&abs, self.target_root)?;
                if let Some(parent) = abs.parent() {
                    fs::create_dir_all(parent).map_err(|source| Error::Io {
                        path: Some(parent.to_path_buf()),
                        source,
                    })?;
                }
                let extension = abs
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                if !needs_render || self.binary_globs.contains(extension.as_str()) {
                    fs::write(&abs, f.contents()).map_err(|source| Error::Io {
                        path: Some(abs.clone()),
                        source,
                    })?;
                } else {
                    let body = std::str::from_utf8(f.contents()).map_err(|_| Error::Render {
                        path: rel.display().to_string(),
                        source: minijinja::Error::new(
                            minijinja::ErrorKind::InvalidOperation,
                            "embedded template contained non-UTF-8 bytes; tag it as a binary extension",
                        ),
                    })?;
                    let rendered = self.render_named(rel.display().to_string(), body)?;
                    let cleaned = strip_skip_lines(&rendered);
                    fs::write(&abs, cleaned).map_err(|source| Error::Io {
                        path: Some(abs.clone()),
                        source,
                    })?;
                }
                self.files_written += 1;
                debug!(path = %rel.display(), "wrote");
                Ok(())
            }
        }
    }

    fn render_str(&self, source: &str) -> Result<String> {
        self.env
            .render_str(source, self.ctx)
            .map_err(|err| Error::PathRender {
                path: source.to_string(),
                source: err,
            })
    }

    fn render_named(&self, name: String, source: &str) -> Result<String> {
        self.env
            .render_str(source, self.ctx)
            .map_err(|err| Error::Render {
                path: name,
                source: err,
            })
    }
}

/// Defense in depth against template-driven path traversal.
///
/// `Path::join` is permissive: passing an absolute path or one with `..` segments on the
/// right-hand side can escape the supposed root. `project_slug` is already validated to
/// `[A-Za-z0-9_]`, but a future template author who reaches for another recipe field in
/// a path needs this guard so the worst case is a render error rather than an arbitrary
/// file write. Both `candidate` and `root` are normalized the *same* way — absolutized
/// against the current directory, then `.`/`..` folded lexically — before comparing, so a
/// relative output dir (the default `-o .`) doesn't produce a spurious `./root` vs `root`
/// mismatch. We never canonicalize on the filesystem, so an attacker-planted symlink in
/// the output tree cannot widen the comparison.
fn guard_path_within_root(candidate: &Path, root: &Path) -> Result<()> {
    let resolved = normalize_lexically(candidate);
    let canon_root = normalize_lexically(root);
    if !resolved.starts_with(&canon_root) {
        return Err(Error::InvalidRecipe(format!(
            "rendered path {} resolves outside the target {}",
            resolved.display(),
            canon_root.display()
        )));
    }
    Ok(())
}

/// Make `path` absolute relative to the current directory (without touching the target
/// filesystem or following symlinks), then collapse `.` and `..` segments lexically. A
/// `..` that would climb past the filesystem root is clamped at the root rather than
/// escaping — the subsequent `starts_with` check is what rejects out-of-root paths.
fn normalize_lexically(path: &Path) -> PathBuf {
    use std::path::Component;

    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };

    let mut normalized: Vec<Component<'_>> = Vec::new();
    for comp in absolute.components() {
        match comp {
            Component::ParentDir => {
                if !matches!(
                    normalized.last(),
                    None | Some(Component::RootDir) | Some(Component::Prefix(_))
                ) {
                    normalized.pop();
                }
            }
            Component::CurDir => {}
            other => normalized.push(other),
        }
    }
    normalized.iter().collect()
}

fn strip_jinja_ext(name: &str) -> String {
    let s = name.strip_suffix(".j2").unwrap_or(name);
    // Dotfile-shadow convention: a leading `_dot_` in a templated filename becomes
    // a literal `.` in the rendered output. Lets template authors check in files
    // named `_dot_gitignore`, `_dot_env.example`, etc. without git in OUR workspace
    // picking them up as real dotfiles.
    if let Some(rest) = s.strip_prefix("_dot_") {
        return format!(".{rest}");
    }
    s.to_string()
}

/// Any line whose entire trimmed content is `__SKIP__` is removed from the rendered output —
/// allows templates to insert sentinels mid-file without leaving stray markers behind.
fn strip_skip_lines(rendered: &str) -> String {
    static_skip_re()
        .replace_all(rendered, "")
        .to_string()
}

fn static_skip_re() -> &'static Regex {
    use std::sync::OnceLock;
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*__SKIP__\s*\n?").unwrap())
}

#[cfg(test)]
mod path_guard_tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn allows_paths_inside_root() {
        let root = Path::new("/tmp/proj");
        assert!(guard_path_within_root(Path::new("/tmp/proj/a/b.py"), root).is_ok());
        assert!(guard_path_within_root(Path::new("/tmp/proj/a/./b"), root).is_ok());
        assert!(guard_path_within_root(Path::new("/tmp/proj/a/x/../b"), root).is_ok());
    }

    #[test]
    fn rejects_dotdot_escaping_root() {
        let root = Path::new("/tmp/proj");
        assert!(guard_path_within_root(Path::new("/tmp/proj/../etc/passwd"), root).is_err());
        assert!(guard_path_within_root(Path::new("/tmp/proj/a/../../etc"), root).is_err());
    }

    #[test]
    fn rejects_absolute_path_outside_root() {
        let root = Path::new("/tmp/proj");
        assert!(guard_path_within_root(Path::new("/etc/passwd"), root).is_err());
        assert!(guard_path_within_root(Path::new("/tmp/other"), root).is_err());
    }

    #[test]
    fn allows_relative_root_with_leading_dot() {
        // The default `-o .` yields `target_root = ./<slug>` and candidates like
        // `./<slug>/LICENSE`. Both sides must normalize identically so the leading
        // `.` doesn't make an in-root path look like an escape.
        let root = Path::new("./awesomeapp");
        assert!(guard_path_within_root(Path::new("./awesomeapp/LICENSE"), root).is_ok());
        assert!(guard_path_within_root(Path::new("./awesomeapp/config/settings/base.py"), root).is_ok());
        assert!(guard_path_within_root(Path::new("awesomeapp/LICENSE"), root).is_ok());
    }

    #[test]
    fn rejects_relative_dotdot_escaping_relative_root() {
        let root = Path::new("./awesomeapp");
        assert!(guard_path_within_root(Path::new("./awesomeapp/../etc/passwd"), root).is_err());
        assert!(guard_path_within_root(Path::new("./awesomeapp/a/../../secret"), root).is_err());
    }
}
