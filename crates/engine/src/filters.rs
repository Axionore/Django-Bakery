//! Custom minijinja filters available inside templates.
//!
//! Mirrors the small set cookiecutter-django relies on, plus a few new conveniences.

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
