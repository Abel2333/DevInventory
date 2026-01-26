#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("config error: {0}")]
    Config(#[from] ConfigError),

    #[error("key error: {0}")]
    Key(#[from] KeyError),

    #[error("crypto error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("failed to read config file")]
    Read(#[source] std::io::Error),

    #[error("failed to parse config file")]
    Parse(#[source] toml::de::Error),

    #[error("cannot determine config directory")]
    ConfigDir,

    #[error("invalid config value: {0}")]
    InvalidValue(String),
}

#[derive(thiserror::Error, Debug)]
pub enum KeyError {
    #[error("master key not found (provide --dmk or set env var)")]
    Missing,

    #[error("invalid base64 master key")]
    InvalidBase64(#[source] base64::DecodeError),

    #[error("invalid master key length (expected 32 bytes, got {0})")]
    InvalidLength(usize),

    #[error("failed to read environment variable: {0}")]
    EnvVar(#[source] std::env::VarError),
}

#[derive(thiserror::Error, Debug)]
pub enum CryptoError {
    #[error("encrypt failed")]
    Encrypt,

    #[error("decrypt failed")]
    Decrypt,

    #[error("ciphertext too short")]
    CiphertextTooShort,
}

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("database connection failed")]
    Connect(#[source] sqlx::Error),

    #[error("migration failed")]
    Migrate(#[source] sqlx::Error),

    #[error("query failed")]
    Query(#[source] sqlx::Error),

    #[error("crypto error")]
    Crypto(#[from] CryptoError),

    #[error("io error")]
    Io(#[source] std::io::Error),
}
