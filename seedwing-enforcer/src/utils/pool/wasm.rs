use super::PoolError;
use std::future::Future;
use tokio::sync::oneshot;

#[derive(Clone, Debug)]
pub struct Pool {}

impl Pool {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn spawn_pinned<F, Fut>(&self, f: F) -> Result<Fut::Output, PoolError>
    where
        F: FnOnce() -> Fut + 'static,
        F: Send + 'static,
        Fut: Future + 'static,
        Fut::Output: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = tx.send(f().await);
        });

        rx.await.map_err(|_err| PoolError)
    }
}
