use crate::backend::notification::{
    FinishOperation, FinishOperationParameters, StartOperation, StartOperationParameters,
    UpdateOperation, UpdateOperationParameters,
};
use async_trait::async_trait;
use seedwing_enforcer_common::utils::progress::ProgressRunner;
use std::{future::Future, pin::Pin, time::Instant};
use tokio::spawn;
use tower_lsp::Client;

pub struct Progress<'a> {
    client: &'a Client,
    token: &'a str,
}

impl Progress<'_> {
    pub async fn update<M, I>(&self, message: M, increment: I)
    where
        M: Into<Option<String>>,
        I: Into<Option<usize>>,
    {
        self.client
            .send_notification::<UpdateOperation>(UpdateOperationParameters {
                token: self.token.to_string(),
                message: message.into(),
                increment: increment.into(),
            })
            .await;
    }
}

#[async_trait(?Send)]
pub trait ProgressExt {
    async fn run_with<T, F, Fut, R>(&self, title: T, total: usize, f: F) -> R
    where
        T: Into<String>,
        F: FnOnce(Progress) -> Fut,
        Fut: Future<Output = R>;
}

#[async_trait(?Send)]
impl ProgressExt for Client {
    async fn run_with<T, F, Fut, R>(&self, title: T, total: usize, f: F) -> R
    where
        T: Into<String>,
        F: FnOnce(Progress) -> Fut,
        Fut: Future<Output = R>,
    {
        run_operation(self.clone(), title, total, f).await
    }
}

pub async fn run_operation<T, F, Fut, R>(client: Client, title: T, total: usize, f: F) -> R
where
    T: Into<String>,
    F: FnOnce(Progress) -> Fut,
    Fut: Future<Output = R>,
{
    let token = uuid::Uuid::new_v4().to_string();

    client
        .send_notification::<StartOperation>(StartOperationParameters {
            token: token.clone(),
            title: title.into(),
            total,
        })
        .await;

    let p = Progress {
        client: &client,
        token: &token,
    };

    let start = Instant::now();

    let result = f(p).await;

    let diff = Instant::now() - start;
    log::info!("Building took: {diff:?}");

    client
        .send_notification::<FinishOperation>(FinishOperationParameters { token })
        .await;

    result
}

pub struct ClientProgress(pub Client);

pub struct ClientProgressRunner {
    client: Client,
    token: String,
}

impl seedwing_enforcer_common::utils::progress::Progress for ClientProgress {
    type Progress = ClientProgressRunner;

    fn start(
        &self,
        title: impl Into<String>,
        total: usize,
    ) -> Pin<Box<dyn Future<Output = Self::Progress>>> {
        let token = uuid::Uuid::new_v4().to_string();
        let title = title.into();
        let client = self.0.clone();
        Box::pin(async move {
            client
                .send_notification::<StartOperation>(StartOperationParameters {
                    token: token.clone(),
                    title,
                    total,
                })
                .await;
            ClientProgressRunner { client, token }
        })
    }
}

impl ProgressRunner for ClientProgressRunner {
    fn update(
        &self,
        message: Option<impl Into<String>>,
        increment: impl Into<Option<usize>>,
    ) -> Pin<Box<dyn Future<Output = ()>>> {
        let client = self.client.clone();
        let message = message.map(|s| s.into());
        let increment = increment.into();
        let token = self.token.clone();

        Box::pin(async move {
            client
                .send_notification::<UpdateOperation>(UpdateOperationParameters {
                    token,
                    message,
                    increment,
                })
                .await;
        })
    }
}

impl Drop for ClientProgressRunner {
    fn drop(&mut self) {
        let client = self.client.clone();
        let token = self.token.clone();
        spawn(async move {
            client
                .send_notification::<FinishOperation>(FinishOperationParameters { token })
                .await;
        });
    }
}
