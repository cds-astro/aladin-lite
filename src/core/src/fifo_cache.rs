use std::collections::{HashMap, VecDeque};

pub struct Cache<K, V> {
    data: HashMap<K, V>,
    order: VecDeque<K>,
}

const SIZE_RESOURCE_CACHE: usize = 1024;
use crate::Abort;
use std::hash::Hash;
impl<K, V> Cache<K, V>
where
    K: Clone + std::cmp::Eq + Hash,
{
    pub fn new() -> Self {
        let data = HashMap::with_capacity(SIZE_RESOURCE_CACHE);
        let order = VecDeque::with_capacity(SIZE_RESOURCE_CACHE);
        Cache { data, order }
    }

    pub fn insert(&mut self, key: K, val: V) {
        if self.order.len() == SIZE_RESOURCE_CACHE {
            let k = self.order.pop_front().unwrap_abort();
            self.data.remove(&k);
        }

        self.data.insert(key.clone(), val);
        self.order.push_back(key);
    }

    pub fn extract_new(&mut self) -> Option<V> {
        if let Some(k) = self.order.pop_back() {
            self.data.remove(&k)
        } else {
            None
        }
    }

    pub fn contains(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }
}
