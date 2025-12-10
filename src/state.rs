use sqlx::PgPool;
use tokio::sync::broadcast;

pub struct AppState {
    pub db_pool: PgPool,
    pub shutdown_tx: broadcast::Sender<()>,
}
