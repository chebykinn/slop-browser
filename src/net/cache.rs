use std::collections::HashMap;
use std::time::{Duration, Instant};

struct CacheEntry {
    data: Vec<u8>,
    expires: Instant,
}

pub struct Cache {
    entries: HashMap<String, CacheEntry>,
    max_entries: usize,
    default_ttl: Duration,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            max_entries: 100,
            default_ttl: Duration::from_secs(300),
        }
    }

    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.entries.get(key).and_then(|entry| {
            if entry.expires > Instant::now() {
                Some(entry.data.as_slice())
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, key: String, data: Vec<u8>) {
        self.insert_with_ttl(key, data, self.default_ttl);
    }

    pub fn insert_with_ttl(&mut self, key: String, data: Vec<u8>, ttl: Duration) {
        if self.entries.len() >= self.max_entries {
            self.evict_expired();
        }

        self.entries.insert(
            key,
            CacheEntry {
                data,
                expires: Instant::now() + ttl,
            },
        );
    }

    pub fn remove(&mut self, key: &str) {
        self.entries.remove(key);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    fn evict_expired(&mut self) {
        let now = Instant::now();
        self.entries.retain(|_, entry| entry.expires > now);
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
