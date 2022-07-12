use std::collections::{HashMap, VecDeque};

pub struct Cache<K, V> {
    data: HashMap<K, V>,
    order: VecDeque<K>,
}

const SIZE_RESOURCE_CACHE: usize = 1024;

use std::hash::Hash;
impl<K, V> Cache<K, V>
where
    K: Clone + std::cmp::Eq + Hash
{
    pub fn new() -> Self {
        let data = HashMap::with_capacity(SIZE_RESOURCE_CACHE);
        let order = VecDeque::with_capacity(SIZE_RESOURCE_CACHE);
        Cache {
            data,
            order
        }
    }

    pub fn insert(&mut self, key: K, val: V) {
        if self.order.len() == SIZE_RESOURCE_CACHE {
            let k = self.order.pop_front().unwrap();
            self.data.remove(&k);
        }

        self.data.insert(key.clone(), val);
        self.order.push_back(key);
    }

    pub fn extract(&mut self, key: &K) -> Option<V> {
        self.data.remove(key)
    }

    pub fn contains(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }
}