const INITIAL_NBUCKETS: usize = 1;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    fn bucket(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % self.buckets.len() as u64) as usize
    }

    fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_NBUCKETS,
            n => 2 * n,
        };
        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Vec::new()));

        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let bucket = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[bucket].push((key, value));
        }

        mem::replace(&mut self.buckets, new_buckets);
    }

    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            items: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items == 0
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let bucket = self.bucket(key);
        self.buckets[bucket]
            .iter()
            .find(|&(ref ekey, _)| ekey == key)
            .map(|&(_, ref v)| v)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }

        let bucket = self.bucket(&key);
        let bucket = &mut self.buckets[bucket];

        for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
            if *ekey == key {
                return Some(mem::replace(evalue, value));
            }
        }

        self.items += 1;
        bucket.push((key, value));
        None
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let bucket = self.bucket(key);
        let bucket = &mut self.buckets[bucket];
        let i = bucket.iter().position(|&(ref ekey, _)| ekey == key)?;
        self.items -= 1;
        Some(bucket.swap_remove(i).1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_hashmap() {
        let map: HashMap<String, String> = HashMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn insert() {
        let mut map: HashMap<String, String> = HashMap::new();
        let res = map.insert("key".to_string(), "value".to_string());
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
        assert!(res.is_none());
        let res = map.insert("key".to_string(), "value".to_string());
        assert_eq!(res.unwrap(), "value".to_string());
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn get() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), 50);
        let res = map.get(&"key".to_string());
        assert_eq!(res, Some(&50));
    }
    #[test]
    fn remove() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), 50);
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
        let res = map.remove(&"key".to_string());
        assert_eq!(res, Some(50));
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }
}
