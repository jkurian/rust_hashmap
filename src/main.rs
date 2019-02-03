pub mod hashmap;
pub mod hashset;

pub fn main() {
    let mut map = hashmap::HashMap::new();
    map.insert("key", "value");

    let mut set = hashset::HashSet::new();
    set.insert(52);
    set.is_empty();
    set.get(52);
    set.len();
}
