//! Console output helpers — banners, summaries, success messages.

use console::{Style, style};
use django_bakery_engine::{Recipe, RenderReport};

pub fn banner() {
    let title = Style::new().for_stdout().cyan().bold();
    let dim = Style::new().for_stdout().dim();
    println!();
    println!("{}", title.apply_to("  🥧  django-bakery"));
    println!(
        "  {}",
        dim.apply_to(format!(
            "v{}  ·  Rust-powered Django scaffolding",
            env!("CARGO_PKG_VERSION")
        ))
    );
    println!("  {}", dim.apply_to("Preheating the oven — let's bake a Django app."));
    println!();
}

pub fn print_summary(r: &Recipe) {
    println!(
        "\n{}  Generating with this recipe:",
        style("▸").cyan().bold()
    );
    row("project", &r.project_name);
    row("slug", &r.project_slug);
    row(
        "stack",
        &format!(
            "Python {} · Django {} · {} · {}",
            r.python_version.as_str(),
            r.django_version.as_str(),
            r.relational_db.as_str(),
            r.api_layer.as_str()
        ),
    );
    row("frontend", r.frontend.as_str());
    row(
        "addons",
        &format!(
            "{}{}{}{}{}",
            tick(r.use_celery, "celery "),
            tick(r.use_sentry, "sentry "),
            tick(r.use_observability, "obs "),
            tick(r.use_pre_commit, "precommit "),
            tick(r.use_mailpit, "mailpit"),
        ),
    );
    println!();
}

pub fn print_success(r: &Recipe, report: &RenderReport) {
    let ok = style("✔").green().bold();
    let croissant = console::Emoji("🥐  ", "");
    println!();
    println!(
        "{}  {}{} {} ({} files, {} directories, {} ms)",
        ok,
        croissant,
        style("Fresh out of the oven —").bold(),
        style(report.project_root.display()).bold(),
        report.files_written,
        report.directories_created,
        report.elapsed_ms
    );
    println!();
    println!("  {}", style("Next steps:").bold());
    println!("    cd {}", r.project_slug);
    if r.use_pre_commit {
        println!("    uv sync && uv run pre-commit install");
    } else {
        println!("    uv sync");
    }
    match r.container_setup {
        django_bakery_engine::ContainerSetup::ComposeTraefik
        | django_bakery_engine::ContainerSetup::ComposeOnly => {
            println!("    docker compose -f compose.local.yml up --build");
        }
        django_bakery_engine::ContainerSetup::None => {
            // The custom AUTH_USER_MODEL ships without a committed initial migration, so
            // `makemigrations` must run before `migrate` — otherwise Django aborts with
            // "Dependency on app with no migrations: users". Mirrors the compose start
            // script (compose/local/django/start.j2).
            if r.multi_tenant {
                println!("    uv run python manage.py makemigrations users tenants");
                println!("    uv run python manage.py migrate_schemas --shared");
                println!("    uv run python manage.py bootstrap_public_tenant");
            } else {
                println!("    uv run python manage.py makemigrations users");
                println!("    uv run python manage.py migrate");
                println!("    uv run python manage.py createsuperuser");
            }
            println!("    uv run python manage.py runserver");
        }
    }
    println!();
    println!(
        "  {} https://github.com/Axionore/Django-Bakery",
        style("Docs:").bold()
    );
    println!();
}

pub fn print_step(msg: &str) {
    eprintln!("  {}  {}", style("↻").yellow(), style(msg).dim());
}

pub fn print_compat(warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }
    let warn = style("⚠").yellow().bold();
    eprintln!();
    eprintln!("  {} {}", warn, style("Compatibility warnings:").bold());
    for w in warnings {
        eprintln!("    {}  {}", style("•").yellow(), w);
    }
}

pub fn print_options() {
    let r = django_bakery_engine::Recipe::defaults();
    let pretty = toml::to_string_pretty(&r).expect("recipe is serializable");
    println!("# Default django-bakery recipe (TOML)");
    println!("# Edit and pass with: django-bakery bake --config recipe.toml --output ./out");
    println!();
    println!("{pretty}");
}

fn row(label: &str, value: &str) {
    println!(
        "  {:>10}  {}",
        style(label).cyan(),
        style(value).bold()
    );
}

fn tick(on: bool, label: &str) -> String {
    if on {
        format!("{} ", style(label).green())
    } else {
        format!("{} ", style(label).dim())
    }
}
