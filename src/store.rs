// TODO in memory backed Store implementation using std::collections::HashMap;
//
// Store should be a trait with methods:
//   get(key) -> string
//   set(key, value)
//
//   and later I guess delete(...) and range(...)
//

// use std::collections::HashMap;

// pub trait Storage<K,V,E> {
//     fn get(&self, key: &K) -> Result<Option<V>, E>;
//     fn set(&self, key: &K, value: &V) -> Result<Option<V>, E>;
// }

// pub struct MemStore<K,V> {
//     store: HashMap<K,V>
// }

// impl<K,V> MemStore<K,V> {
//     pub fn new() -> MemStore<K, V> {
//         MemStore { store: HashMap::new() }
//     }

// }

// impl<K,V,E> Storage<K,V,E> for MemStore<K,V> {
//     fn get(&self, key: &K) -> Result<Option<V>, E> {
//     //     error[E0599]: the method `get` exists for struct `HashMap<K, V>`, but its trait bounds were not satisfied
//     //     --> src/store.rs:30:20
//     //      |
//     //   30 |         self.store.get(&key)
//     //      |                    ^^^ method cannot be called on `HashMap<K, V>` due to unsatisfied trait bounds
//         self.store.get(&key)
//     }

//     fn set(&self, key: &K, value: &V) -> Result<Option<V>, E> {
//         todo!()
//     }
// }
