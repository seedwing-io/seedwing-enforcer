use std::{future::Future, pin::Pin};

pub trait ProgressRunner {
    fn update(
        &self,
        message: Option<impl Into<String>>,
        increment: impl Into<Option<usize>>,
    ) -> Pin<Box<dyn Future<Output = ()>>>;
}

pub trait Progress: Send {
    type Progress: ProgressRunner;

    fn start(
        &self,
        title: impl Into<String>,
        total: usize,
    ) -> Pin<Box<dyn Future<Output = Self::Progress>>>;
}

pub struct NoProgress;

impl Progress for NoProgress {
    type Progress = ();

    fn start(
        &self,
        _title: impl Into<String>,
        _total: usize,
    ) -> Pin<Box<dyn Future<Output = Self::Progress>>> {
        Box::pin(async {})
    }
}

impl ProgressRunner for () {
    fn update(
        &self,
        _message: Option<impl Into<String>>,
        _increment: impl Into<Option<usize>>,
    ) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(async {})
    }
}
