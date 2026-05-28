//! The Recipe — the full set of user choices for one project generation.
//!
//! A Recipe is the JSON-/TOML-serializable shape consumed by the engine. The CLI
//! constructs it interactively (via prompts) or non-interactively (via `--config file.toml`).

use serde::{Deserialize, Serialize};

/// Top-level recipe captured at generation time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    // --- basics -----------------------------------------------------------
    pub project_name: String,
    pub project_slug: String,
    pub description: String,
    pub author_name: String,
    pub author_email: String,
    pub domain_name: String,
    pub license: License,
    pub open_source: bool,

    // --- stack ------------------------------------------------------------
    pub python_version: PythonVersion,
    pub django_version: DjangoVersion,
    pub mode: Mode,
    pub relational_db: RelationalDb,
    pub graph_db: GraphDb,
    pub api_layer: ApiLayer,
    pub frontend: Frontend,
    pub radix_flavor: Option<RadixFlavor>,
    pub frontend_variant: FrontendVariant,
    pub js_language: JsLanguage,
    pub js_testing: bool,
    pub css_framework: CssFramework,

    // --- add-ons ----------------------------------------------------------
    pub use_celery: bool,
    pub celery_broker: CeleryBroker,
    pub use_mailpit: bool,
    pub prod_email: ProdEmail,
    pub storage: Storage,
    pub use_sentry: bool,
    pub use_observability: bool,
    pub use_feature_flags: bool,
    pub type_checker: TypeChecker,
    pub use_pre_commit: bool,
    pub ci: CiProvider,
    pub container_setup: ContainerSetup,
    pub version_control: VersionControl,
    pub timezone: String,

    /// Multi-tenant mode — when `true`, scaffolds `django-tenants` with
    /// PG-schema-per-tenant isolation, splits INSTALLED_APPS into
    /// SHARED_APPS + TENANT_APPS, and adds an `apps/tenants/` app with
    /// Tenant + Domain models and a `create_tenant` management command.
    /// Forces `relational_db = postgres` (django-tenants is PG-only —
    /// it uses native Postgres schemas).
    #[serde(default)]
    pub multi_tenant: bool,
}

impl Recipe {
    /// Build a Recipe pre-filled with sensible production defaults. The CLI overlays user
    /// answers on top of this.
    pub fn defaults() -> Self {
        Self {
            project_name: "My Awesome App".into(),
            project_slug: "my_awesome_app".into(),
            description: "A production-grade Django project generated with django-bakery.".into(),
            author_name: "Your Name".into(),
            author_email: "you@example.com".into(),
            domain_name: "example.com".into(),
            license: License::Mit,
            open_source: true,
            python_version: PythonVersion::Py314,
            django_version: DjangoVersion::Dj60,
            mode: Mode::Production,
            relational_db: RelationalDb::Postgres,
            graph_db: GraphDb::None,
            api_layer: ApiLayer::Ninja,
            frontend: Frontend::HtmxAlpine,
            radix_flavor: None,
            frontend_variant: FrontendVariant::Full,
            js_language: JsLanguage::Typescript,
            js_testing: true,
            css_framework: CssFramework::Tailwind,
            use_celery: true,
            celery_broker: CeleryBroker::Redis,
            use_mailpit: true,
            prod_email: ProdEmail::AnymailMailgun,
            storage: Storage::AwsS3,
            use_sentry: true,
            use_observability: true,
            use_feature_flags: false,
            type_checker: TypeChecker::Mypy,
            use_pre_commit: true,
            ci: CiProvider::GitHubActions,
            container_setup: ContainerSetup::ComposeTraefik,
            version_control: VersionControl::Git,
            timezone: "UTC".into(),
            multi_tenant: false,
        }
    }

    pub fn is_postgres(&self) -> bool { matches!(self.relational_db, RelationalDb::Postgres) }
    pub fn is_sqlite(&self) -> bool { matches!(self.relational_db, RelationalDb::Sqlite) }
    pub fn is_mysqlish(&self) -> bool {
        matches!(self.relational_db, RelationalDb::Mysql | RelationalDb::Mariadb)
    }

    /// Validates the recipe semantics beyond what serde checks (slug shape, allowed combos).
    pub fn validate(&self) -> Result<(), String> {
        if self.project_slug.is_empty() {
            return Err("project_slug must not be empty".into());
        }
        let first = self.project_slug.chars().next().unwrap();
        if !first.is_ascii_alphabetic() && first != '_' {
            return Err("project_slug must start with an ASCII letter or underscore".into());
        }
        if !self
            .project_slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err("project_slug may only contain ASCII letters, digits, and underscores".into());
        }
        if !self.author_email.contains('@') {
            return Err("author_email must contain '@'".into());
        }
        if self.multi_tenant && !self.is_postgres() {
            return Err(
                "multi_tenant requires relational_db = 'postgres' (django-tenants is PG-only — \
                 it uses native Postgres schemas for tenant isolation)".into(),
            );
        }
        match self.frontend {
            Frontend::React
                if matches!(self.frontend_variant, FrontendVariant::Full)
                    && self.radix_flavor.is_none() =>
            {
                return Err(
                    "radix_flavor is required when frontend is 'react' and variant is 'full'"
                        .into(),
                );
            }
            _ => {}
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Enum payloads — kept as kebab/lower-case strings in TOML for readability.
// ---------------------------------------------------------------------------

macro_rules! str_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $s:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum $name { $(#[serde(rename = $s)] $variant),+ }

        impl $name {
            pub const fn as_str(&self) -> &'static str {
                match self { $(Self::$variant => $s),+ }
            }
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
        }
    };
}

str_enum! {
    /// Open-source license offered at generation time.
    License {
        Mit => "mit",
        Apache2 => "apache-2",
        Bsd3 => "bsd-3",
        Proprietary => "proprietary",
    }
}

str_enum! {
    /// Python interpreter version to target in generated projects.
    PythonVersion {
        Py314 => "3.14",
        Py313 => "3.13",
    }
}

impl PythonVersion {
    pub fn dotted(&self) -> &'static str { self.as_str() }
    pub fn short(&self) -> &'static str { match self { Self::Py314 => "314", Self::Py313 => "313" } }
}

str_enum! {
    /// Django framework version.
    DjangoVersion {
        Dj60 => "6.0",
    }
}

str_enum! {
    /// Which environment's defaults to optimize for at generation time.
    Mode {
        Production => "production",
        Development => "development",
    }
}

str_enum! {
    /// Primary relational database.
    RelationalDb {
        Postgres => "postgres",
        Sqlite => "sqlite",
        Mysql => "mysql",
        Mariadb => "mariadb",
    }
}

str_enum! {
    /// Optional graph database add-on.
    GraphDb {
        None => "none",
        Neo4j => "neo4j",
        Nebula => "nebula",
        Surreal => "surreal",
        Dgraph => "dgraph",
    }
}

str_enum! {
    /// API layer style.
    ApiLayer {
        Ninja => "ninja",
        Drf => "drf",
        GraphqlStrawberry => "graphql-strawberry",
        GraphqlGraphene => "graphql-graphene",
        None => "none",
    }
}

str_enum! {
    /// Frontend style for the generated project.
    Frontend {
        HtmxAlpine => "htmx-alpine",
        React => "react",
        Nuxt => "nuxt",
        Vue => "vue",
        Next => "next",
        DjangoTemplates => "django-templates",
        None => "none",
    }
}

str_enum! {
    /// Whether the SPA scaffold is opinion-free (skeleton) or comes wired with
    /// auth, routing, UI library, and state management (full template).
    FrontendVariant {
        Full => "full",
        Skeleton => "skeleton",
    }
}

str_enum! {
    /// Radix flavor when the React frontend is chosen.
    RadixFlavor {
        Themes => "themes",
        Primitives => "primitives",
    }
}

str_enum! {
    /// JavaScript dialect for SPA frontends.
    JsLanguage {
        Typescript => "typescript",
        Javascript => "javascript",
    }
}

str_enum! {
    /// CSS framework for server-rendered templates.
    CssFramework {
        Tailwind => "tailwind",
        Bootstrap => "bootstrap",
        None => "none",
    }
}

str_enum! {
    /// Celery broker (only used when `use_celery` is true).
    CeleryBroker {
        Redis => "redis",
        Rabbitmq => "rabbitmq",
    }
}

str_enum! {
    /// Production email backend.
    ProdEmail {
        AnymailMailgun   => "anymail-mailgun",
        AnymailSes       => "anymail-ses",
        AnymailSendgrid  => "anymail-sendgrid",
        AnymailMailjet   => "anymail-mailjet",
        AnymailMandrill  => "anymail-mandrill",
        AnymailPostmark  => "anymail-postmark",
        AnymailBrevo     => "anymail-brevo",
        AnymailSparkpost => "anymail-sparkpost",
        Smtp             => "smtp",
        Console          => "console",
    }
}

str_enum! {
    /// Cloud storage backend for media/static assets in production.
    Storage {
        AwsS3 => "aws-s3",
        Gcs => "gcs",
        AzureBlob => "azure-blob",
        Whitenoise => "whitenoise",
        Nginx => "nginx",
        None => "none",
    }
}

str_enum! {
    /// Static type checker.
    TypeChecker {
        Mypy => "mypy",
        Pyright => "pyright",
        None => "none",
    }
}

str_enum! {
    /// CI provider templates to emit.
    CiProvider {
        GitHubActions => "github-actions",
        GitLabCi => "gitlab-ci",
        Both => "both",
        None => "none",
    }
}

str_enum! {
    /// Container setup to emit.
    ContainerSetup {
        ComposeTraefik => "compose-traefik",
        ComposeOnly => "compose-only",
        None => "none",
    }
}

str_enum! {
    /// Version control system to initialize in the generated project.
    VersionControl {
        Git => "git",
        Jj => "jj",
        Hg => "hg",
        None => "none",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        Recipe::defaults().validate().expect("defaults must validate");
    }

    #[test]
    fn validate_rejects_bad_slug_start() {
        let mut r = Recipe::defaults();
        r.project_slug = "9bad".into();
        assert!(r.validate().is_err());
    }

    #[test]
    fn validate_rejects_slug_with_dash() {
        let mut r = Recipe::defaults();
        r.project_slug = "has-dash".into();
        assert!(r.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_slug() {
        let mut r = Recipe::defaults();
        r.project_slug = "".into();
        assert!(r.validate().is_err());
    }

    #[test]
    fn validate_rejects_email_without_at() {
        let mut r = Recipe::defaults();
        r.author_email = "no-at-sign".into();
        assert!(r.validate().is_err());
    }

    #[test]
    fn validate_requires_radix_flavor_when_react() {
        let mut r = Recipe::defaults();
        r.frontend = Frontend::React;
        r.radix_flavor = None;
        assert!(r.validate().is_err());

        r.radix_flavor = Some(RadixFlavor::Themes);
        assert!(r.validate().is_ok());
    }

    #[test]
    fn toml_roundtrip_preserves_human_strings() {
        let r = Recipe::defaults();
        let s = toml::to_string(&r).expect("serialize");
        // Enum values are rendered as their human-friendly strings, not their variant idents.
        assert!(s.contains("django_version = \"6.0\""));
        assert!(s.contains("license = \"mit\""));
        assert!(s.contains("api_layer = \"ninja\""));

        let back: Recipe = toml::from_str(&s).expect("deserialize");
        assert_eq!(back.django_version, DjangoVersion::Dj60);
        assert_eq!(back.license, License::Mit);
        assert_eq!(back.api_layer, ApiLayer::Ninja);
        assert_eq!(back.relational_db, r.relational_db);
        assert_eq!(back.project_slug, r.project_slug);
    }

    #[test]
    fn db_helpers_match_recipe() {
        let mut r = Recipe::defaults();
        r.relational_db = RelationalDb::Postgres;
        assert!(r.is_postgres());
        assert!(!r.is_sqlite());
        assert!(!r.is_mysqlish());

        r.relational_db = RelationalDb::Sqlite;
        assert!(r.is_sqlite());
        assert!(!r.is_postgres());

        r.relational_db = RelationalDb::Mariadb;
        assert!(r.is_mysqlish());
    }

    #[test]
    fn python_version_helpers() {
        assert_eq!(PythonVersion::Py314.dotted(), "3.14");
        assert_eq!(PythonVersion::Py314.short(), "314");
        assert_eq!(PythonVersion::Py313.short(), "313");
    }
}
