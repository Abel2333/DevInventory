use std::path::{Path, PathBuf};

use crate::{
    crypto::{CryptoService, MasterKey},
    error::AppError,
    keymgr::MasterKeyProvider,
    storage::Repository,
};

pub struct InitResult {
    pub db_path: PathBuf,
    pub master_key: MasterKey,
}

pub async fn init(
    db_path: &Path,
    key_provider: &MasterKeyProvider,
) -> Result<InitResult, AppError> {
    let repo = Repository::connect(db_path).await?;
    repo.migrate().await?;

    let key_result = key_provider.obtain(true)?;
    let crypto_service = CryptoService::new(key_result.into_key());
    let master_key = crypto_service.master_key().clone();

    Ok(InitResult {
        db_path: db_path.to_path_buf(),
        master_key,
    })
}
