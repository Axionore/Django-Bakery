//! `django-bakery` CLI entry point.

mod prompts;
mod recipe_io;
mod ui;

use std::path::PathBuf;

use anyhow::{Context as _, Result};
use clap::{Parser, Subcommand};
use django_bakery_engine::{RenderOptions, render};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(
    name = "django-bakery",
    version,
    about = "A fast, modern Django project generator written in Rust.",
    long_about = "django-bakery generates production-grade Django 6 projects with Postgres, Docker, \
                  Django Ninja or DRF, HTMX+Alpine+Tailwind v4 frontends, Celery, Sentry, structlog, \
                  CI, and OWASP-aligned defaults baked in. Everything you'd expect from \
                  cookiecutter-django — faster, in a single binary, with better defaults for 2026."
)]
struct Cli {
    /// Increase log verbosity (-v, -vv, -vvv).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Generate a new project interactively (default).
    New(NewArgs),
    /// Render a project non-interactively from a saved recipe file.
    Bake(BakeArgs),
    /// Print a JSON-schema-ish summary of every recipe option.
    Options,
    /// Validate a recipe file without rendering.
    Validate {
        /// Path to the recipe (TOML or JSON).
        path: PathBuf,
    },
    /// Print an example recipe file you can edit and re-use with `bake`.
    Sample {
        /// Output format.
        #[arg(long, value_enum, default_value_t = SampleFormat::Toml)]
        format: SampleFormat,
    },
}

#[derive(Debug, clap::Args)]
struct NewArgs {
    /// Output directory (parent of the generated project). Defaults to the current directory.
    #[arg(short, long, default_value = ".")]
    output: PathBuf,
    /// Overwrite the target project directory if non-empty.
    #[arg(long)]
    force: bool,
    /// Do not initialize a VCS in the generated project.
    #[arg(long, alias = "no-git")]
    no_vcs: bool,
    /// Run `uv sync` (and friends) inside the generated project after creation.
    #[arg(long)]
    bootstrap: bool,
    /// Skip all interactive prompts and accept every default.
    #[arg(long)]
    yes: bool,
    /// Pre-fill the recipe from this file before prompting (any answer in the file is the
    /// new default).
    #[arg(long)]
    preset: Option<PathBuf>,
    /// Skip the PyPI/npm latest-version check; use bundled defaults only.
    #[arg(long)]
    offline: bool,
    /// Treat any compatibility-check warning as a hard error.
    #[arg(long)]
    strict_compat: bool,
}

#[derive(Debug, clap::Args)]
struct BakeArgs {
    /// Path to the recipe file (TOML or JSON).
    #[arg(short, long)]
    config: PathBuf,
    /// Output directory.
    #[arg(short, long, default_value = ".")]
    output: PathBuf,
    /// Overwrite the target project directory if non-empty.
    #[arg(long)]
    force: bool,
    /// Skip VCS init.
    #[arg(long, alias = "no-git")]
    no_vcs: bool,
    /// Run `uv sync` after generation.
    #[arg(long)]
    bootstrap: bool,
    /// Skip the PyPI/npm latest-version check; use bundled defaults only.
    #[arg(long)]
    offline: bool,
    /// Treat any compatibility-check warning as a hard error.
    #[arg(long)]
    strict_compat: bool,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum SampleFormat {
    Toml,
    Json,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    match cli.command {
        Cmd::New(args) => cmd_new(args),
        Cmd::Bake(args) => cmd_bake(args),
        Cmd::Options => {
            ui::print_options();
            Ok(())
        }
        Cmd::Validate { path } => cmd_validate(path),
        Cmd::Sample { format } => cmd_sample(format),
    }
}

fn cmd_new(args: NewArgs) -> Result<()> {
    ui::banner();

    let preset = match args.preset.as_ref() {
        Some(p) => Some(recipe_io::load(p)?),
        None => None,
    };
    let recipe = if args.yes {
        preset.unwrap_or_else(django_bakery_engine::Recipe::defaults)
    } else {
        prompts::interactive(preset.as_ref()).context("interactive prompts cancelled")?
    };
    recipe.validate().map_err(|e| anyhow::anyhow!(e))?;

    let mut opts = RenderOptions::new(&args.output);
    opts.force = args.force;
    opts.git_init = !args.no_vcs;
    opts.bootstrap = args.bootstrap;
    opts.version_mode = if args.offline {
        django_bakery_engine::ResolveMode::Offline
    } else {
        django_bakery_engine::ResolveMode::Online
    };
    opts.strict_compat = args.strict_compat;

    ui::print_summary(&recipe);
    if !args.offline {
        ui::print_step("Checking PyPI / npm for the latest stable versions…");
    }
    let report = render(&recipe, &opts).context("render failed")?;
    ui::print_compat(&report.compat_warnings);
    ui::print_success(&recipe, &report);
    Ok(())
}

fn cmd_bake(args: BakeArgs) -> Result<()> {
    let recipe = recipe_io::load(&args.config)?;
    recipe.validate().map_err(|e| anyhow::anyhow!(e))?;
    let mut opts = RenderOptions::new(&args.output);
    opts.force = args.force;
    opts.git_init = !args.no_vcs;
    opts.bootstrap = args.bootstrap;
    opts.version_mode = if args.offline {
        django_bakery_engine::ResolveMode::Offline
    } else {
        django_bakery_engine::ResolveMode::Online
    };
    opts.strict_compat = args.strict_compat;
    let report = render(&recipe, &opts).context("render failed")?;
    ui::print_compat(&report.compat_warnings);
    ui::print_success(&recipe, &report);
    Ok(())
}

fn cmd_validate(path: PathBuf) -> Result<()> {
    let recipe = recipe_io::load(&path)?;
    recipe.validate().map_err(|e| anyhow::anyhow!(e))?;
    println!("{} {}", console::style("✓").green().bold(), "recipe is valid");
    Ok(())
}

fn cmd_sample(format: SampleFormat) -> Result<()> {
    let recipe = django_bakery_engine::Recipe::defaults();
    let body = match format {
        SampleFormat::Toml => toml::to_string_pretty(&recipe)?,
        SampleFormat::Json => serde_json::to_string_pretty(&recipe)?,
    };
    println!("{body}");
    Ok(())
}

fn init_tracing(verbose: u8) {
    let level = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level)))
        .with_target(false)
        .compact()
        .try_init();
}
