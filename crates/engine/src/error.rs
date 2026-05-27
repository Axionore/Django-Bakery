use std::io;
use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("output directory {0} already exists and is not empty; pass --force to overwrite")]
    OutputDirNotEmpty(PathBuf),

    #[error("invalid recipe: {0}")]
    InvalidRecipe(String),

    #[error("template render error in {path}: {source}")]
    Render {
        path: String,
        #[source]
        source: minijinja::Error,
    },

    #[error("template path render error in {path}: {source}")]
    PathRender {
        path: String,
        #[source]
        source: minijinja::Error,
    },

    #[error("recipe parse error: {0}")]
    RecipeParse(String),

    #[error("io error at {path:?}: {source}")]
    Io {
        path: Option<PathBuf>,
        #[source]
        source: io::Error,
    },
}

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        Self::Io { path: None, source }
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Self::RecipeParse(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::RecipeParse(value.to_string())
    }
}
