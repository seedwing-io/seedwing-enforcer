use async_trait::async_trait;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter},
    path::PathBuf,
    sync::Arc,
};
use tokio::sync::Mutex;
use tower_lsp::{lsp_types::Diagnostic, Client};
use url::Url;

#[async_trait]
pub trait Target: Send + Sync {
    async fn publish(&self, uri: Url, diags: Vec<Diagnostic>);
}

#[async_trait]
impl Target for Client {
    async fn publish(&self, uri: Url, diags: Vec<Diagnostic>) {
        self.publish_diagnostics(uri, diags, None).await
    }
}

#[derive(Clone, Debug, Default)]
pub struct MockTarget(Arc<Mutex<Vec<(Url, Vec<Diagnostic>)>>>);

#[async_trait]
impl Target for MockTarget {
    async fn publish(&self, uri: Url, diags: Vec<Diagnostic>) {
        self.0.lock().await.push((uri, diags));
    }
}

#[allow(dead_code)]
impl MockTarget {
    pub async fn clear(&self) {
        self.0.lock().await.clear();
    }

    pub async fn get(&self) -> Vec<(Url, Vec<Diagnostic>)> {
        self.0.lock().await.clone()
    }

    pub async fn drain(&self) -> Vec<(Url, Vec<Diagnostic>)> {
        self.0.lock().await.drain(..).collect()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Category {
    Enforcer,
    Source,
}

pub struct DiagnosticPublisher {
    target: Box<dyn Target>,
    state: HashMap<Url, HashMap<Category, Vec<Diagnostic>>>,
}

impl Debug for DiagnosticPublisher {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiagnosticPublisher")
            .field("target", &"<...>")
            .field("state", &self.state)
            .finish()
    }
}

impl DiagnosticPublisher {
    pub fn new(target: impl Target + 'static) -> Self {
        Self {
            target: Box::new(target),
            state: Default::default(),
        }
    }

    #[allow(unused)]
    pub async fn publish_list(&mut self, category: Category, list: Vec<(Url, Diagnostic)>) {
        let mut map = HashMap::new();
        for (k, v) in list {
            map.entry(k).or_insert_with(Vec::new).push(v);
        }

        self.publish(category, map).await;
    }

    pub async fn publish_file(
        &mut self,
        category: Category,
        map: HashMap<PathBuf, Vec<Diagnostic>>,
    ) {
        self.publish(
            category,
            map.into_iter()
                .filter_map(|(k, v)| Url::from_file_path(k).ok().map(|u| (u, v)))
                .collect(),
        )
        .await;
    }

    pub async fn publish(&mut self, category: Category, map: HashMap<Url, Vec<Diagnostic>>) {
        let mut changed = HashSet::new();

        // remove missing
        for (url, state) in &mut self.state {
            if !map.contains_key(url) {
                state.remove(&category);
                changed.insert(url.clone());
            }
        }

        // set new
        for (url, diags) in map {
            self.state
                .entry(url.clone())
                .or_default()
                .insert(category, diags);
            changed.insert(url.clone());
        }

        // publish changes
        for (url, state) in &self.state {
            let diags: Vec<_> = state.iter().flat_map(|(_, e)| e.iter().cloned()).collect();
            log::info!("Publishing diagnostics: {}: {}", url, diags.len());
            self.target.publish(url.clone(), diags).await;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::backend::project::publisher::{Category, DiagnosticPublisher, MockTarget};
    use std::collections::HashMap;
    use tower_lsp::lsp_types::Diagnostic;
    use url::Url;

    #[tokio::test]
    async fn test_1() {
        let target = MockTarget::default();
        let mut publisher = DiagnosticPublisher::new(target.clone());

        let f1 = Url::from_file_path("/foo").unwrap();

        publisher
            .publish(
                Category::Enforcer,
                HashMap::from([(
                    f1.clone(),
                    vec![Diagnostic {
                        message: "foo".into(),
                        ..Default::default()
                    }],
                )]),
            )
            .await;

        assert_eq!(
            target.drain().await,
            vec![(
                f1.clone(),
                vec![Diagnostic {
                    message: "foo".into(),
                    ..Default::default()
                }]
            )]
        );

        publisher
            .publish(Category::Enforcer, HashMap::default())
            .await;

        assert_eq!(target.drain().await, vec![(f1.clone(), vec![])]);
    }
}
