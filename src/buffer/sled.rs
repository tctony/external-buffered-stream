use std::sync::atomic::{AtomicU64, Ordering};

use crate::{Error, ExternalBufferSerde};

use super::ExternalBuffer;

/// Sled as the persistent buffer with FIFO queue order
pub struct ExternalBufferSled {
    db: sled::Db,
    head_counter: AtomicU64,
    tail_counter: AtomicU64,
}

impl ExternalBufferSled {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        let db = sled::open(path)?;

        // Initialize counters by scanning existing keys
        let (head, tail) = Self::initialize_counters(&db)?;

        Ok(Self {
            db,
            head_counter: AtomicU64::new(head),
            tail_counter: AtomicU64::new(tail),
        })
    }

    fn initialize_counters(db: &sled::Db) -> Result<(u64, u64), Error> {
        let mut min_key = u64::MAX;
        let mut max_key = 0u64;
        let mut has_keys = false;

        for result in db.iter() {
            let (key, _) = result?;
            if key.len() == 8 {
                let key_u64 = u64::from_be_bytes(
                    key.as_ref()
                        .try_into()
                        .map_err(|_| Error::InvalidSledKeyFormat)?,
                );
                min_key = min_key.min(key_u64);
                max_key = max_key.max(key_u64);
                has_keys = true;
            }
        }

        if has_keys {
            Ok((min_key, max_key + 1))
        } else {
            Ok((0, 0))
        }
    }

    fn key_from_u64(value: u64) -> [u8; 8] {
        value.to_be_bytes()
    }
}

impl<T: ExternalBufferSerde> ExternalBuffer<T> for ExternalBufferSled {
    fn push(&self, item: T) -> Result<(), Error> {
        let serialized = item.into_external_buffer()?;
        let key = self.tail_counter.fetch_add(1, Ordering::SeqCst);
        let key_bytes = Self::key_from_u64(key);

        self.db.insert(&key_bytes, serialized)?;
        Ok(())
    }

    fn shift(&self) -> Result<Option<T>, Error> {
        loop {
            let current_head = self.head_counter.load(Ordering::SeqCst);
            let current_tail = self.tail_counter.load(Ordering::SeqCst);

            // Check if buffer is empty
            if current_head >= current_tail {
                return Ok(None);
            }

            let key_bytes = Self::key_from_u64(current_head);

            // Try to remove the item atomically
            match self.db.remove(&key_bytes)? {
                Some(data) => {
                    // Successfully removed, update head counter
                    self.head_counter.fetch_add(1, Ordering::SeqCst);

                    // Deserialize and return the item
                    let item = T::from_external_buffer(&data)?;
                    return Ok(Some(item));
                }
                None => {
                    // Item was already removed by another thread, try next
                    self.head_counter.fetch_add(1, Ordering::SeqCst);
                    continue;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bincode::{Decode, Encode};
    use tempfile::TempDir;

    #[derive(Debug, Clone, PartialEq, Encode, Decode)]
    struct TestItem {
        id: u32,
        name: String,
    }

    #[test]
    fn test_push_and_shift() {
        let temp_dir = TempDir::new().unwrap();
        let buffer = ExternalBufferSled::new(temp_dir.path().join("test_db")).unwrap();

        let item1 = TestItem {
            id: 1,
            name: "first".to_string(),
        };
        let item2 = TestItem {
            id: 2,
            name: "second".to_string(),
        };

        // Push items
        buffer.push(item1.clone()).unwrap();
        buffer.push(item2.clone()).unwrap();

        // Shift items (should come out in FIFO order)
        let shifted1 = buffer.shift().unwrap();
        assert_eq!(shifted1, Some(item1));

        let shifted2 = buffer.shift().unwrap();
        assert_eq!(shifted2, Some(item2));

        // Buffer should be empty now
        let shifted3: Option<TestItem> = buffer.shift().unwrap();
        assert_eq!(shifted3, None);
    }

    #[test]
    fn test_empty_buffer() {
        let temp_dir = TempDir::new().unwrap();
        let buffer = ExternalBufferSled::new(temp_dir.path().join("empty_db")).unwrap();

        // Empty buffer should return None
        let result: Option<TestItem> = buffer.shift().unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("persistent_db");

        let item = TestItem {
            id: 42,
            name: "persistent".to_string(),
        };

        // Create buffer, push item, and drop it
        {
            let buffer = ExternalBufferSled::new(&db_path).unwrap();
            buffer.push(item.clone()).unwrap();
        }

        // Create new buffer with same path and verify item is still there
        {
            let buffer = ExternalBufferSled::new(&db_path).unwrap();
            let retrieved = buffer.shift().unwrap();
            assert_eq!(retrieved, Some(item));
        }
    }

    #[test]
    fn test_multiple_pushes_and_shifts() {
        let temp_dir = TempDir::new().unwrap();
        let buffer = ExternalBufferSled::new(temp_dir.path().join("multi_db")).unwrap();

        let items: Vec<TestItem> = (0..10)
            .map(|i| TestItem {
                id: i,
                name: format!("item_{}", i),
            })
            .collect();

        // Push all items
        for item in &items {
            buffer.push(item.clone()).unwrap();
        }

        // Shift all items and verify order
        for expected_item in &items {
            let shifted = buffer.shift().unwrap();
            assert_eq!(shifted, Some(expected_item.clone()));
        }

        // Buffer should be empty
        let result: Option<TestItem> = buffer.shift().unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_interleaved_push_and_shift() {
        let temp_dir = TempDir::new().unwrap();
        let buffer = ExternalBufferSled::new(temp_dir.path().join("interleaved_db")).unwrap();

        let item1 = TestItem {
            id: 1,
            name: "first".to_string(),
        };
        let item2 = TestItem {
            id: 2,
            name: "second".to_string(),
        };
        let item3 = TestItem {
            id: 3,
            name: "third".to_string(),
        };

        // Push one, shift one
        buffer.push(item1.clone()).unwrap();
        let shifted1 = buffer.shift().unwrap();
        assert_eq!(shifted1, Some(item1));

        // Push two, shift two
        buffer.push(item2.clone()).unwrap();
        buffer.push(item3.clone()).unwrap();

        let shifted2 = buffer.shift().unwrap();
        assert_eq!(shifted2, Some(item2));

        let shifted3 = buffer.shift().unwrap();
        assert_eq!(shifted3, Some(item3));

        // Should be empty
        let result: Option<TestItem> = buffer.shift().unwrap();
        assert_eq!(result, None);
    }
}
