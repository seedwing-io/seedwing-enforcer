use super::PoolError;
use std::future::Future;
use tokio_util::task::LocalPoolHandle;

#[derive(Clone, Debug)]
pub struct Pool {
    pool: LocalPoolHandle,
}

impl Pool {
    pub fn new() -> Self {
        Self {
            pool: LocalPoolHandle::new(8),
        }
    }

    pub async fn spawn_pinned<F, Fut>(&self, f: F) -> Result<Fut::Output, PoolError>
    where
        F: FnOnce() -> Fut,
        F: Send + 'static,
        Fut: Future + 'static,
        Fut::Output: Send + 'static,
    {
        self.pool.spawn_pinned(f).await.map_err(|_err| PoolError)
    }
}

impl Default for Pool {
    fn default() -> Self {
        Self::new()
    }
}
