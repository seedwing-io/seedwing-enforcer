use crate::enforcer::Dependency;
use seedwing_policy_engine::runtime::Response;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A cache for dependency evaluations
pub trait Cache: Send {
    fn get(&self, dependency: &Dependency) -> Option<Response>;
    fn store(&self, dependency: &Dependency, response: Response);
}

pub struct NoCache;

impl Cache for NoCache {
    fn get(&self, _: &Dependency) -> Option<Response> {
        None
    }

    fn store(&self, _: &Dependency, _: Response) {}
}

#[derive(Clone, Debug, Default)]
pub struct DefaultCache {
    store: Arc<RwLock<HashMap<String, Response>>>,
}

impl DefaultCache {
    pub fn invalidate(&self) {
        self.store.write().unwrap().clear();
    }
}

impl Cache for DefaultCache {
    fn get(&self, dependency: &Dependency) -> Option<Response> {
        self.store
            .read()
            .unwrap()
            .get(dependency.cache_key())
            .cloned()
    }

    fn store(&self, dependency: &Dependency, response: Response) {
        self.store
            .write()
            .unwrap()
            .insert(dependency.cache_key().to_string(), response);
    }
}
