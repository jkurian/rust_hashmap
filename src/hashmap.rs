const INITIAL_NBUCKETS: usize = 1;
use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}

pub struct OccupiedEntry<'a, K, V> {
    element: &'a mut (K, V),
}

pub struct VacantEntry<'a, K, V> {
    key: K,
    map: &'a mut HashMap<K, V>,
    bucket: usize,
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    pub fn insert(self, value: V) -> &'a mut V
    where
        K: Hash + Eq,
    {
        if self.map.buckets.is_empty() || self.map.items > 3 * self.map.buckets.len() / 4 {
            self.map.resize();
        }

        self.map.buckets[self.bucket].push((self.key, value));
        self.map.items += 1;

        &mut self.map.buckets[self.bucket].last_mut().unwrap().1
    }
}

pub enum Entry<'a, K, V> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Hash + Eq,
{
    pub fn or_insert(self, value: V) -> &'a mut V {
        match self {
            Entry::Occupied(e) => &mut e.element.1,
            Entry::Vacant(e) => e.insert(value),
        }
    }
    pub fn or_insert_with<F>(self, maker: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Entry::Occupied(e) => &mut e.element.1,
            Entry::Vacant(e) => e.insert(maker()),
        }
    }
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert_with(Default::default)
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    fn bucket<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
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

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.bucket(key);
        self.buckets[bucket]
            .iter()
            .find(|&(ref ekey, _)| ekey.borrow() == key)
            .map(|&(_, ref v)| v)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).is_some()
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

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.bucket(key);
        let bucket = &mut self.buckets[bucket];
        let i = bucket
            .iter()
            .position(|&(ref ekey, _)| ekey.borrow() == key)?;
        self.items -= 1;
        Some(bucket.swap_remove(i).1)
    }

    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }

        let bucket = self.bucket(&key);
        match self.buckets[bucket]
            .iter_mut()
            .find(|&&mut (ref ekey, _)| ekey == &key)
        {
            Some(entry) => Entry::Occupied(OccupiedEntry {
                element: { unsafe { &mut *(entry as *mut _) } },
            }),
            None => Entry::Vacant(VacantEntry {
                key,
                map: self,
                bucket,
            }),
        }
    }
}

pub struct Iter<'a, K: 'a, V: 'a> {
    map: &'a HashMap<K, V>,
    bucket: usize,
    at: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => match bucket.get(self.at) {
                    Some(&(ref k, ref v)) => {
                        self.at += 1;
                        break Some((k, v));
                    }
                    None => {
                        self.bucket += 1;
                        self.at = 0;
                        continue;
                    }
                },
                None => break None,
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map: self,
            bucket: 0,
            at: 0,
        }
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
    #[test]
    fn iter() {
        let mut map = HashMap::new();
        map.insert("key", 50);
        map.insert("key2", 100);
        map.insert("key3", 150);

        for (&k, &v) in &map {
            match k {
                "key" => assert_eq!(v, 50),
                "key2" => assert_eq!(v, 100),
                "key3" => assert_eq!(v, 150),
                _ => unreachable!(),
            }
        }
        assert_eq!((&map).into_iter().count(), 3);
    }
    #[test]
    fn entry() {
        let mut map = HashMap::new();
        map.entry("abcd").or_insert(50);
        assert_eq!(map.get(&"abcd").unwrap(), &50);
        map.entry("abcd").or_insert(60);
        assert_eq!(map.get(&"abcd").unwrap(), &50);
    }

}
