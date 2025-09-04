use std::collections::HashMap;
use crate::tacky::tacky_symbols::TackyVariable;

pub struct AppendOnlyHashMap<K, V> {
    map: HashMap<K, V>,
}
impl<K: std::hash::Hash + Eq, V> AppendOnlyHashMap<K,V> {
    pub fn new() -> Self {
        AppendOnlyHashMap {
        map: HashMap::new()
    }
    }
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, ()> {
        if self.map.contains_key(&key) {
            Err(())
        } else {
            Ok(self.map.insert(key, value))
        }
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    pub fn to_hash_map(self) -> HashMap<K, V> {
        self.map
    }
    pub fn to_buffered(&self) -> BufferedHashMap<K, V> {
        BufferedHashMap::new(self)
    }
}

pub struct BufferedHashMap<'a, K, V> {
    read_only: &'a AppendOnlyHashMap<K, V>,
    buffer: HashMap<K, V>
}
impl <'a, K: std::hash::Hash + Eq, V> BufferedHashMap<'a, K,V> {
    pub fn new(base: &'a AppendOnlyHashMap<K, V>) -> Self {
        BufferedHashMap {
            read_only: base,
            buffer: HashMap::new()
        }
    }
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, ()> {
        if self.read_only.get(&key).is_some() || self.buffer.contains_key(&key) {
            Err(())
        } else {
            Ok(self.buffer.insert(key, value))
        }
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        if let Some(value) = self.buffer.get(key) {
            Some(value)
        } else {
            self.read_only.get(key)
        }
    }
}


pub struct StackAllocationResult {
    pub new_stack_value: u64,
    // new allocations of variable ids to their stack addresses
    pub new_var_stack_allocations: HashMap<u64, u64>
}
impl StackAllocationResult {
    pub fn new(new_stack_value: u64) -> Self {
        StackAllocationResult {
            new_stack_value,
            new_var_stack_allocations: HashMap::new()
        }
    }
    pub fn with_allocations(
        new_stack_value: u64,
        new_var_stack_allocations: HashMap<u64, u64>
    ) -> Self {
        StackAllocationResult {
            new_stack_value,
            new_var_stack_allocations
        }
    }
}

pub trait ToStackAllocated {
    fn to_stack_allocated(
        &self, stack_value: u64,
        allocations: &AppendOnlyHashMap<u64, u64>
        // returns a tuple of (Self, new stack_value)
    ) -> (Self, StackAllocationResult) where Self: Sized;
}
