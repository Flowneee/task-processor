use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub(crate) struct MemoryDb<T: Clone> {
    inner: Arc<RwLock<HashMap<u64, T>>>,
    id: Arc<RwLock<u64>>,
}

impl<T: Clone> MemoryDb<T> {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            id: Arc::new(RwLock::new(0)),
        }
    }

    pub(crate) fn insert(&self, key: u64, value: T) -> Option<T> {
        // panic here, if `.write()` fails, because we cannot use database anymore
        self.inner.clone().write().unwrap().insert(key, value)
    }

    pub(crate) fn get(&self, key: u64) -> Option<T> {
        // panic here, if `.read()` fails, because we cannot use database anymore
        self.inner.clone().read().unwrap().get(&key).cloned()
    }

    pub(crate) fn delete(&self, key: u64) -> Option<T> {
        // panic here, if `.write()` fails, because we cannot use database anymore
        self.inner.clone().write().unwrap().remove(&key)
    }

    pub(crate) fn generate_id(&self) -> u64 {
        // panic here, if `.write()` fails, because we cannot use database anymore
        let id_arc = self.id.clone();
        let mut id = id_arc.write().unwrap();
        let old_id = *id;
        *id += 1;
        old_id
    }
}

impl<T: serde::Serialize + Clone> MemoryDb<T> {
    pub(crate) fn dump_to_json(&self) -> serde_json::Value {
        let mut obj = serde_json::map::Map::new();
        self.inner
            .clone()
            .read()
            .unwrap()
            .iter()
            .for_each(|(k, v)| {
                obj.insert(k.to_string(), serde_json::to_value(v).unwrap());
            });
        serde_json::Value::Object(obj)
    }
}
