use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct HashSet<V>
where
    V: Hash + Eq,
{
    items: Vec<V>,
}

impl<V> HashSet<V>
where
    V: Hash + Eq,
{
    fn index<Q>(&self, value: &Q) -> usize
    where
        V: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        (hasher.finish() % self.items.len() as u64) as usize
    }
    pub fn new() -> HashSet<V> {
        HashSet { items: Vec::new() }
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.len() == 0
    }

    pub fn insert(&mut self, value: V) -> bool {
        if self.items.len() == 0 {
            self.items.push(value);
            return true;
        }
        let index = self.index(&value);

        if index < self.items.len() && self.items[index] == value {
            return false;
        } else {
            self.items.push(value);
        }

        true
    }

    pub fn get(&self, value: V) -> Option<V> {
        let index = self.index(&value);

        if index < self.items.len() && self.items[index] == value {
            return Some(value);
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_hashset() {
        let set: HashSet<String> = HashSet::new();
        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
    }
    #[test]
    fn insert() {
        let mut set = HashSet::new();
        assert!(set.insert(50));
        assert!(set.insert(70));
        assert_eq!(set.len(), 2);
        assert!(!set.insert(50));
    }

}
