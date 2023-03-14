use crate::enforcer::{Dependency, Outcome};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A cache for dependency evaluations
pub trait Cache: Send {
    fn get(&self, dependency: &Dependency) -> Option<Outcome>;
    fn store(&self, dependency: &Dependency, outcome: Outcome);
}

pub struct NoCache;

impl Cache for NoCache {
    fn get(&self, _dependency: &Dependency) -> Option<Outcome> {
        None
    }

    fn store(&self, _dependency: &Dependency, _outcome: Outcome) {}
}

#[derive(Clone, Debug, Default)]
pub struct DefaultCache {
    store: Arc<RwLock<HashMap<String, Outcome>>>,
}

impl DefaultCache {
    pub fn invalidate(&self) {
        self.store.write().unwrap().clear();
    }
}

impl Cache for DefaultCache {
    fn get(&self, dependency: &Dependency) -> Option<Outcome> {
        self.store
            .read()
            .unwrap()
            .get(dependency.cache_key())
            .cloned()
    }

    fn store(&self, dependency: &Dependency, outcome: Outcome) {
        self.store
            .write()
            .unwrap()
            .insert(dependency.cache_key().to_string(), outcome);
    }
}
