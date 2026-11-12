use std::sync::Arc;

use crate::config::Config;
use crate::db::Db;

/// Shared state handed to every handler.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<Db>,
}

impl AppState {
    pub fn new(config: Config, db: Db) -> Self {
        Self {
            config: Arc::new(config),
            db: Arc::new(db),
        }
    }
}
