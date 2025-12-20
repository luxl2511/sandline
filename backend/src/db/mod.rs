pub mod pool;

pub use pool::create_pool;
pub type DbPool = sqlx::PgPool;
