pub mod hashmap;

pub fn main() {
    let mut map = hashmap::HashMap::new();
    map.insert("key", "value");
}
