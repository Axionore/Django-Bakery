//! Interactive prompts powered by `inquire`. Builds a [`Recipe`] from user input,
//! using a preset (if supplied) as the default for each question.

use std::fmt;

use anyhow::Result;
use django_bakery_engine::{
    ApiLayer, CeleryBroker, CiProvider, Frontend, FrontendVariant, GraphDb, JsLanguage, Mode,
    ProdEmail, RadixFlavor, Recipe, RelationalDb, Storage, TypeChecker, VersionControl, secret_key,
};
use heck::ToSnakeCase;
use inquire::{Confirm, Select, Text};
use inquire::validator::Validation;

/// Build a Recipe by walking the user through grouped prompts.
pub fn interactive(preset: Option<&Recipe>) -> Result<Recipe> {
    let defaults = preset.cloned().unwrap_or_else(Recipe::defaults);
    stage(1);

    let project_name = Text::new("Project name")
        .with_default(&defaults.project_name)
        .with_help_message("Human-friendly title, e.g. 'My Awesome App'")
        .with_validator(|s: &str| {
            if s.trim().is_empty() {
                Ok(Validation::Invalid("required".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()?;

    let slug_default = project_name.to_snake_case();
    let project_slug = Text::new("Project slug")
        .with_default(&slug_default)
        .with_help_message("Python package name; ASCII letters, digits, underscore; starts with letter or _")
        .with_validator(|s: &str| {
            let first = s.chars().next();
            let valid = !s.is_empty()
                && first.map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false)
                && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            Ok(if valid { Validation::Valid } else { Validation::Invalid("invalid slug".into()) })
        })
        .prompt()?;

    let description = Text::new("Short description")
        .with_default(&defaults.description)
        .prompt()?;

    let author_name = Text::new("Author name")
        .with_default(&defaults.author_name)
        .prompt()?;

    let author_email = Text::new("Author email")
        .with_default(&defaults.author_email)
        .with_validator(|s: &str| {
            if s.contains('@') && s.len() > 3 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("must contain '@'".into()))
            }
        })
        .prompt()?;

    let domain_name = Text::new("Primary domain (for prod settings + Traefik)")
        .with_default(&defaults.domain_name)
        .prompt()?;

    let license = pick("License", license_options(), defaults.license)?;
    let open_source = Confirm::new("Is this an open-source project?")
        .with_default(defaults.open_source)
        .prompt()?;

    stage(2);

    let python_version = pick(
        "Python version",
        py_options(),
        defaults.python_version,
    )?;
    let django_version = pick(
        "Django version",
        django_options(),
        defaults.django_version,
    )?;
    let mode = pick(
        "Primary mode",
        mode_options(),
        defaults.mode,
    )?;
    let relational_db = pick(
        "Primary relational database",
        db_options(),
        defaults.relational_db,
    )?;
    let graph_db = pick(
        "Graph database add-on",
        graph_options(),
        defaults.graph_db,
    )?;
    let api_layer = pick(
        "API layer",
        api_options(),
        defaults.api_layer,
    )?;
    let frontend = pick(
        "Frontend",
        frontend_options(),
        defaults.frontend,
    )?;
    let is_spa = matches!(
        frontend,
        Frontend::React | Frontend::Nuxt | Frontend::Vue | Frontend::Next
    );
    let frontend_variant = if is_spa {
        pick("Frontend variant", variant_options(), defaults.frontend_variant)?
    } else {
        FrontendVariant::Full
    };
    let radix_flavor = if frontend == Frontend::React
        && matches!(frontend_variant, FrontendVariant::Full)
    {
        Some(pick(
            "Radix flavor",
            radix_options(),
            defaults.radix_flavor.unwrap_or(RadixFlavor::Themes),
        )?)
    } else {
        None
    };
    let js_language = if is_spa {
        pick("Language", js_language_options(), defaults.js_language)?
    } else {
        defaults.js_language
    };
    let js_testing = if is_spa && matches!(frontend_variant, FrontendVariant::Full) {
        Confirm::new("Wire up Vitest 8 + Playwright?")
            .with_default(defaults.js_testing)
            .prompt()?
    } else {
        false
    };
    let css_framework = pick(
        "CSS framework (server-rendered templates)",
        css_options(),
        defaults.css_framework,
    )?;

    stage(3);

    let use_celery = Confirm::new("Add Celery (+ Beat + Flower)?")
        .with_default(defaults.use_celery)
        .prompt()?;
    let celery_broker = if use_celery {
        pick(
            "Celery broker",
            broker_options(),
            defaults.celery_broker,
        )?
    } else {
        defaults.celery_broker
    };
    let use_mailpit = Confirm::new("Add Mailpit for local email testing?")
        .with_default(defaults.use_mailpit)
        .prompt()?;
    let prod_email = pick(
        "Production email backend",
        prod_email_options(),
        defaults.prod_email,
    )?;
    let storage = pick(
        "Cloud storage backend",
        storage_options(),
        defaults.storage,
    )?;
    let use_sentry = Confirm::new("Add Sentry?")
        .with_default(defaults.use_sentry)
        .prompt()?;
    let use_observability = Confirm::new("Add structlog + OpenTelemetry?")
        .with_default(defaults.use_observability)
        .prompt()?;
    let use_feature_flags = Confirm::new("Add django-waffle feature flags?")
        .with_default(defaults.use_feature_flags)
        .prompt()?;
    let multi_tenant = if matches!(relational_db, RelationalDb::Postgres) {
        Confirm::new("Multi-tenant via django-tenants (PG-schema-per-tenant)?")
            .with_default(defaults.multi_tenant)
            .with_help_message(
                "Adds an `apps/tenants/` app with Tenant + Domain models, a `create_tenant` \
                 management command, and splits INSTALLED_APPS into SHARED_APPS + TENANT_APPS.",
            )
            .prompt()?
    } else {
        false
    };
    let type_checker = pick(
        "Type checker",
        typecheck_options(),
        defaults.type_checker,
    )?;
    let use_pre_commit = Confirm::new("Add pre-commit hooks (ruff, djlint, mypy, gitleaks)?")
        .with_default(defaults.use_pre_commit)
        .prompt()?;
    let ci = pick(
        "CI provider",
        ci_options(),
        defaults.ci,
    )?;
    let container_setup = pick(
        "Container setup",
        container_options(),
        defaults.container_setup,
    )?;
    let version_control = pick(
        "Version control",
        vcs_options(),
        defaults.version_control,
    )?;
    let timezone = Text::new("Timezone (IANA)")
        .with_default(&defaults.timezone)
        .prompt()?;

    // --- Credentials & secrets -------------------------------------------
    // Every secret below is pre-seeded with freshly-generated strong entropy — accept with
    // Enter, or type your own. `.env` is gitignored so these stay local, but we still ship
    // real randomness rather than a memorable default nobody remembers to rotate.
    credentials_banner();

    let db_password = if matches!(relational_db, RelationalDb::Sqlite) {
        String::new() // SQLite is file-based — no server password.
    } else {
        Text::new("Database password")
            .with_default(&secret_key(40))
            .with_help_message("Reused by the database container and DATABASE_URL.")
            .prompt()?
    };

    let superuser_default_email = if author_email.is_empty() {
        "admin@example.com"
    } else {
        author_email.as_str()
    };
    let superuser_email = Text::new("Initial admin (superuser) email")
        .with_default(superuser_default_email)
        .prompt()?;
    let superuser_password = Text::new("Initial admin (superuser) password")
        .with_default(&secret_key(32))
        .with_help_message("Seeded idempotently on first boot via `manage.py seed_superuser`.")
        .prompt()?;

    let (flower_user, flower_password) = if use_celery {
        let user = Text::new("Flower dashboard username")
            .with_default("flower")
            .prompt()?;
        let password = Text::new("Flower dashboard password")
            .with_default(&secret_key(24))
            .prompt()?;
        (user, password)
    } else {
        (String::new(), String::new())
    };

    let redis_password = if use_celery && matches!(celery_broker, CeleryBroker::Redis) {
        Text::new("Redis password")
            .with_default(&secret_key(32))
            .prompt()?
    } else {
        String::new()
    };

    let allowed_hosts = Text::new("Allowed hosts (comma-separated)")
        .with_default(&defaults.allowed_hosts)
        .prompt()?;
    let admin_url_suffix = Text::new("Admin URL suffix (obscures /admin/ from scanners)")
        .with_default(&secret_key(16).to_lowercase())
        .prompt()?;

    Ok(Recipe {
        project_name,
        project_slug,
        description,
        author_name,
        author_email,
        domain_name,
        license,
        open_source,
        python_version,
        django_version,
        mode,
        relational_db,
        graph_db,
        api_layer,
        frontend,
        radix_flavor,
        frontend_variant,
        js_language,
        js_testing,
        css_framework,
        use_celery,
        celery_broker,
        use_mailpit,
        prod_email,
        storage,
        use_sentry,
        use_observability,
        use_feature_flags,
        type_checker,
        use_pre_commit,
        ci,
        container_setup,
        version_control,
        timezone,
        multi_tenant,
        db_password,
        superuser_email,
        superuser_password,
        flower_user,
        flower_password,
        redis_password,
        allowed_hosts,
        admin_url_suffix,
    })
}

/// One-time credentials banner: a bakery pun, then a serious security maxim. Printed once,
/// just before the secret prompts, so the moment lands without nagging on every field.
fn credentials_banner() {
    eprintln!(
        "\n  {}{}",
        console::Emoji("🥐  ", ""),
        console::style("Fresh-baked secrets, still warm — take them as-is or knead your own.")
            .bold(),
    );
    eprintln!(
        "  {}",
        console::style("\"Security is a process, not a product.\" — Bruce Schneier")
            .dim()
            .italic(),
    );
}

/// The interactive flow's stages, in order. The emoji is a small bakery pun that
/// also hints at the stage's meaning: a mixing bowl to start, a *stack* of pancakes
/// for the tech stack, a cherry-on-top for the optional add-ons. `console::Emoji`
/// degrades to nothing on terminals that don't render emoji.
const STAGES: [(console::Emoji<'static, 'static>, &str); 3] = [
    (console::Emoji("🥣 ", ""), "Project basics"),
    (console::Emoji("🥞 ", ""), "Stack"),
    (console::Emoji("🍒 ", ""), "Production add-ons"),
];

/// Render the stepper rail above each section so the user always knows where they
/// are in the branching questionnaire. Discrete `[n/3]` rather than a percentage —
/// the number of questions per stage is conditional (graph DB, Radix flavor, broker
/// only appear sometimes), so a fill-bar would jump around and read as broken.
fn stage(current: usize) {
    let separator = console::style("  →  ").dim().to_string();
    let rail = STAGES
        .iter()
        .enumerate()
        .map(|(index, (emoji, label))| {
            let step = index + 1;
            let chip = format!("{emoji}{label}");
            match step.cmp(&current) {
                std::cmp::Ordering::Equal => console::style(chip).cyan().bold(),
                std::cmp::Ordering::Less => console::style(chip).green().dim(),
                std::cmp::Ordering::Greater => console::style(chip).dim(),
            }
            .to_string()
        })
        .collect::<Vec<_>>()
        .join(&separator);

    eprintln!(
        "\n  {}   {}\n  {}",
        rail,
        console::style(format!("[{current}/{}]", STAGES.len())).dim(),
        console::style("─".repeat(60)).dim(),
    );
}

/// Generic single-select prompt with a typed default. Returns the selected variant.
fn pick<T>(prompt: &str, options: Vec<Labeled<T>>, default: T) -> Result<T>
where
    T: Copy + PartialEq + fmt::Debug,
{
    let starting = options
        .iter()
        .position(|o| o.value == default)
        .unwrap_or(0);
    let chosen = Select::new(prompt, options).with_starting_cursor(starting).prompt()?;
    Ok(chosen.value)
}

struct Labeled<T> {
    value: T,
    label: &'static str,
    hint: &'static str,
}

impl<T> fmt::Display for Labeled<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.hint.is_empty() {
            write!(f, "{}", self.label)
        } else {
            write!(f, "{} — {}", self.label, console::style(self.hint).dim())
        }
    }
}

// --- option tables -----------------------------------------------------------

use django_bakery_engine::{DjangoVersion, License, PythonVersion, CssFramework, ContainerSetup};

fn license_options() -> Vec<Labeled<License>> {
    vec![
        Labeled { value: License::Mit, label: "MIT", hint: "permissive, widely used" },
        Labeled { value: License::Apache2, label: "Apache 2.0", hint: "permissive with patent grant" },
        Labeled { value: License::Bsd3, label: "BSD-3-Clause", hint: "permissive" },
        Labeled { value: License::Proprietary, label: "Proprietary", hint: "all rights reserved" },
    ]
}

fn py_options() -> Vec<Labeled<PythonVersion>> {
    vec![
        Labeled { value: PythonVersion::Py314, label: "Python 3.14", hint: "default (2026)" },
        Labeled { value: PythonVersion::Py313, label: "Python 3.13", hint: "for older infra" },
    ]
}

fn django_options() -> Vec<Labeled<DjangoVersion>> {
    vec![Labeled { value: DjangoVersion::Dj60, label: "Django 6.0", hint: "LTS-track, async ORM" }]
}

fn mode_options() -> Vec<Labeled<Mode>> {
    vec![
        Labeled { value: Mode::Production, label: "Production", hint: "Postgres + Docker by default" },
        Labeled { value: Mode::Development, label: "Development", hint: "SQLite by default, fast iteration" },
    ]
}

fn db_options() -> Vec<Labeled<RelationalDb>> {
    vec![
        Labeled { value: RelationalDb::Postgres, label: "PostgreSQL 18", hint: "recommended" },
        Labeled { value: RelationalDb::Sqlite, label: "SQLite", hint: "zero-config dev" },
        Labeled { value: RelationalDb::Mysql, label: "MySQL 8", hint: "Oracle MySQL" },
        Labeled { value: RelationalDb::Mariadb, label: "MariaDB 11", hint: "MySQL fork" },
    ]
}

fn graph_options() -> Vec<Labeled<GraphDb>> {
    vec![
        Labeled { value: GraphDb::None, label: "None", hint: "no graph DB" },
        Labeled { value: GraphDb::Neo4j, label: "Neo4j 5", hint: "Cypher, neomodel" },
        Labeled { value: GraphDb::Nebula, label: "NebulaGraph", hint: "distributed, nGQL" },
        Labeled { value: GraphDb::Surreal, label: "SurrealDB", hint: "multi-model" },
        Labeled { value: GraphDb::Dgraph, label: "Dgraph", hint: "GraphQL+DQL" },
    ]
}

fn api_options() -> Vec<Labeled<ApiLayer>> {
    vec![
        Labeled { value: ApiLayer::Ninja, label: "Django Ninja", hint: "OpenAPI, async, Pydantic 2" },
        Labeled { value: ApiLayer::Drf, label: "Django REST Framework", hint: "battle-tested, full-featured" },
        Labeled { value: ApiLayer::GraphqlStrawberry, label: "GraphQL (Strawberry)", hint: "type-first" },
        Labeled { value: ApiLayer::GraphqlGraphene, label: "GraphQL (Graphene)", hint: "classic" },
        Labeled { value: ApiLayer::None, label: "None", hint: "no API layer" },
    ]
}

fn frontend_options() -> Vec<Labeled<Frontend>> {
    vec![
        Labeled { value: Frontend::HtmxAlpine, label: "HTMX + Alpine.js", hint: "server-rendered, minimal JS" },
        Labeled { value: Frontend::React, label: "React + Vite", hint: "SPA, optional Radix UI" },
        Labeled { value: Frontend::Nuxt, label: "Nuxt 4", hint: "Vue + Nitro, SSR" },
        Labeled { value: Frontend::Vue, label: "Vue 3 + Vite", hint: "SPA, no Nuxt opinions" },
        Labeled { value: Frontend::Next, label: "Next.js 16", hint: "React + App Router, SSR" },
        Labeled { value: Frontend::DjangoTemplates, label: "Django templates only", hint: "no JS framework" },
        Labeled { value: Frontend::None, label: "None (API-only)", hint: "headless backend" },
    ]
}

fn variant_options() -> Vec<Labeled<FrontendVariant>> {
    vec![
        Labeled {
            value: FrontendVariant::Full,
            label: "Full template",
            hint: "auth wired, UI library, router, state — production-ready",
        },
        Labeled {
            value: FrontendVariant::Skeleton,
            label: "Skeleton",
            hint: "minimal scaffold + Django integration — build your own from here",
        },
    ]
}

fn radix_options() -> Vec<Labeled<RadixFlavor>> {
    vec![
        Labeled { value: RadixFlavor::Themes, label: "Radix Themes", hint: "pre-styled components" },
        Labeled { value: RadixFlavor::Primitives, label: "Radix Primitives + Tailwind v4", hint: "headless + custom styles" },
    ]
}

fn js_language_options() -> Vec<Labeled<JsLanguage>> {
    vec![
        Labeled { value: JsLanguage::Typescript, label: "TypeScript 6+", hint: "default — strict typing" },
        Labeled { value: JsLanguage::Javascript, label: "JavaScript (ESM)", hint: "no type checker" },
    ]
}

fn vcs_options() -> Vec<Labeled<VersionControl>> {
    vec![
        Labeled { value: VersionControl::Git, label: "git", hint: "default; `git init --initial-branch=main`" },
        Labeled { value: VersionControl::Jj, label: "jj (Jujutsu)", hint: "git-colocated" },
        Labeled { value: VersionControl::Hg, label: "Mercurial (hg)", hint: "" },
        Labeled { value: VersionControl::None, label: "None", hint: "no VCS init" },
    ]
}

fn css_options() -> Vec<Labeled<CssFramework>> {
    vec![
        Labeled { value: CssFramework::Tailwind, label: "Tailwind v4", hint: "CSS-first, @theme tokens" },
        Labeled { value: CssFramework::Bootstrap, label: "Bootstrap 5", hint: "classic, batteries-included" },
        Labeled { value: CssFramework::None, label: "None", hint: "bring your own CSS" },
    ]
}

fn broker_options() -> Vec<Labeled<CeleryBroker>> {
    vec![
        Labeled { value: CeleryBroker::Redis, label: "Redis", hint: "fast, simple" },
        Labeled { value: CeleryBroker::Rabbitmq, label: "RabbitMQ", hint: "robust messaging" },
    ]
}

fn prod_email_options() -> Vec<Labeled<ProdEmail>> {
    vec![
        Labeled { value: ProdEmail::AnymailMailgun,   label: "Anymail + Mailgun",   hint: "" },
        Labeled { value: ProdEmail::AnymailSes,       label: "Anymail + AWS SES",   hint: "" },
        Labeled { value: ProdEmail::AnymailSendgrid,  label: "Anymail + SendGrid",  hint: "" },
        Labeled { value: ProdEmail::AnymailMailjet,   label: "Anymail + Mailjet",   hint: "" },
        Labeled { value: ProdEmail::AnymailMandrill,  label: "Anymail + Mandrill",  hint: "" },
        Labeled { value: ProdEmail::AnymailPostmark,  label: "Anymail + Postmark",  hint: "" },
        Labeled { value: ProdEmail::AnymailBrevo,     label: "Anymail + Brevo",     hint: "ex-Sendinblue" },
        Labeled { value: ProdEmail::AnymailSparkpost, label: "Anymail + SparkPost", hint: "" },
        Labeled { value: ProdEmail::Smtp,             label: "SMTP",                hint: "any provider" },
        Labeled { value: ProdEmail::Console,          label: "Console (dev only)",  hint: "print to stdout" },
    ]
}

fn storage_options() -> Vec<Labeled<Storage>> {
    vec![
        Labeled { value: Storage::AwsS3, label: "AWS S3", hint: "" },
        Labeled { value: Storage::Gcs, label: "Google Cloud Storage", hint: "" },
        Labeled { value: Storage::AzureBlob, label: "Azure Blob Storage", hint: "" },
        Labeled { value: Storage::Whitenoise, label: "WhiteNoise (static only)", hint: "no media uploads" },
        Labeled { value: Storage::Nginx, label: "nginx-served media", hint: "self-hosted" },
        Labeled { value: Storage::None, label: "None", hint: "Django defaults / local filesystem" },
    ]
}

fn typecheck_options() -> Vec<Labeled<TypeChecker>> {
    vec![
        Labeled { value: TypeChecker::Mypy, label: "mypy", hint: "with django-stubs" },
        Labeled { value: TypeChecker::Pyright, label: "pyright", hint: "Microsoft's, faster" },
        Labeled { value: TypeChecker::None, label: "None", hint: "" },
    ]
}

fn ci_options() -> Vec<Labeled<CiProvider>> {
    vec![
        Labeled { value: CiProvider::GitHubActions, label: "GitHub Actions", hint: "default" },
        Labeled { value: CiProvider::GitLabCi, label: "GitLab CI", hint: "" },
        Labeled { value: CiProvider::Both, label: "Both (GitHub + GitLab)", hint: "" },
        Labeled { value: CiProvider::None, label: "None", hint: "" },
    ]
}

fn container_options() -> Vec<Labeled<ContainerSetup>> {
    vec![
        Labeled { value: ContainerSetup::ComposeTraefik, label: "Docker Compose + Traefik + Let's Encrypt", hint: "full prod" },
        Labeled { value: ContainerSetup::ComposeOnly, label: "Docker Compose (no Traefik)", hint: "behind your own LB" },
        Labeled { value: ContainerSetup::None, label: "None", hint: "no containers" },
    ]
}
