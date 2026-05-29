//! Custom minijinja filters available inside templates.
//!
//! Slugify / case helpers plus engine-side conveniences (random secret_key etc.).

use heck::{ToKebabCase, ToSnakeCase, ToUpperCamelCase};
use minijinja::{Environment, Error, Value};

use crate::context::secret_key;

pub fn register(env: &mut Environment<'_>) {
    env.add_filter("slugify", slugify);
    env.add_filter("snake_case", snake_case);
    env.add_filter("kebab_case", kebab_case);
    env.add_filter("camel_case", camel_case);
    env.add_filter("py_module", snake_case);
    env.add_filter("env_var", env_var);
    env.add_function("secret_key", secret_key_fn);
}

fn slugify(value: String) -> String {
    value.to_kebab_case()
}

fn snake_case(value: String) -> String {
    value.to_snake_case()
}

fn kebab_case(value: String) -> String {
    value.to_kebab_case()
}

fn camel_case(value: String) -> String {
    value.to_upper_camel_case()
}

fn env_var(value: String) -> String {
    value.to_uppercase().replace(['-', '.'], "_")
}

fn secret_key_fn(len: Option<usize>) -> Result<Value, Error> {
    Ok(Value::from(secret_key(len.unwrap_or(50))))
}

#[cfg(test)]
mod tests {
    use minijinja::Environment;

    fn env_with_filters() -> Environment<'static> {
        let mut env = Environment::new();
        super::register(&mut env);
        env
    }

    #[test]
    fn slugify_filter_produces_kebab() {
        let env = env_with_filters();
        let out = env
            .render_str("{{ 'My Awesome App' | slugify }}", ())
            .unwrap();
        assert_eq!(out, "my-awesome-app");
    }

    #[test]
    fn snake_case_filter_handles_spaces_and_camel() {
        let env = env_with_filters();
        assert_eq!(
            env.render_str("{{ 'My Awesome App' | snake_case }}", ())
                .unwrap(),
            "my_awesome_app"
        );
        assert_eq!(
            env.render_str("{{ 'AcmeCorp' | snake_case }}", ()).unwrap(),
            "acme_corp"
        );
    }

    #[test]
    fn kebab_case_filter() {
        let env = env_with_filters();
        assert_eq!(
            env.render_str("{{ 'MyAwesomeApp' | kebab_case }}", ())
                .unwrap(),
            "my-awesome-app"
        );
    }

    #[test]
    fn camel_case_filter() {
        let env = env_with_filters();
        assert_eq!(
            env.render_str("{{ 'my_awesome_app' | camel_case }}", ())
                .unwrap(),
            "MyAwesomeApp"
        );
    }

    #[test]
    fn env_var_filter_uppercases_and_normalizes() {
        let env = env_with_filters();
        assert_eq!(
            env.render_str("{{ 'my-app.name' | env_var }}", ()).unwrap(),
            "MY_APP_NAME"
        );
    }

    #[test]
    fn secret_key_function_default_length() {
        let env = env_with_filters();
        let out = env.render_str("{{ secret_key() }}", ()).unwrap();
        assert_eq!(out.len(), 50);
    }

    #[test]
    fn secret_key_function_custom_length() {
        let env = env_with_filters();
        let out = env.render_str("{{ secret_key(16) }}", ()).unwrap();
        assert_eq!(out.len(), 16);
    }

    #[test]
    fn secret_key_is_non_deterministic() {
        let env = env_with_filters();
        let a = env.render_str("{{ secret_key(32) }}", ()).unwrap();
        let b = env.render_str("{{ secret_key(32) }}", ()).unwrap();
        assert_ne!(a, b, "two consecutive secret_key() calls must differ");
    }
}
