use std::collections::BinaryHeap;
use std::sync::Mutex;

use crate::Error;

use super::ExternalBuffer;

/// A in memory max binary heap queue as the buffer
pub struct ExternalBufferQueue<T: Ord> {
    queue: Mutex<BinaryHeap<T>>,
}

impl<T: Ord> ExternalBufferQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: Default::default(),
        }
    }
}

impl<T: Ord + Send> ExternalBuffer<T> for ExternalBufferQueue<T> {
    fn push(&self, item: T) -> Result<(), Error> {
        let mut queue = self.queue.lock()?;
        queue.push(item);
        Ok(())
    }

    fn shift(&self) -> Result<Option<T>, Error> {
        let mut queue = self.queue.lock()?;
        Ok(queue.pop())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct TestItem {
        priority: i32,
        id: u32,
        name: String,
    }

    impl TestItem {
        fn new(priority: i32, id: u32, name: &str) -> Self {
            Self {
                priority,
                id,
                name: name.to_string(),
            }
        }
    }

    #[test]
    fn test_new_queue_is_empty() {
        let buffer = ExternalBufferQueue::<i32>::new();
        assert!(buffer.shift().unwrap().is_none());
    }

    #[test]
    fn test_push_and_shift_single_item() {
        let buffer = ExternalBufferQueue::new();

        buffer.push(42).unwrap();
        assert_eq!(buffer.shift().unwrap(), Some(42));
        assert!(buffer.shift().unwrap().is_none());
    }

    #[test]
    fn test_push_and_shift_multiple_items() {
        let buffer = ExternalBufferQueue::new();

        let item1 = TestItem::new(1, 1, "low priority");
        let item2 = TestItem::new(5, 2, "high priority");
        let item3 = TestItem::new(3, 3, "medium priority");

        // Push items
        buffer.push(item1.clone()).unwrap();
        buffer.push(item2.clone()).unwrap();
        buffer.push(item3.clone()).unwrap();

        // Should get items in max-heap order (highest priority first)
        assert_eq!(buffer.shift().unwrap(), Some(item2)); // priority 5
        assert_eq!(buffer.shift().unwrap(), Some(item3)); // priority 3
        assert_eq!(buffer.shift().unwrap(), Some(item1)); // priority 1
        assert!(buffer.shift().unwrap().is_none());
    }

    #[test]
    fn test_max_heap_behavior() {
        let buffer = ExternalBufferQueue::new();

        // Push numbers in arbitrary order
        let numbers = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
        for num in &numbers {
            buffer.push(*num).unwrap();
        }

        let mut result = Vec::new();
        while let Some(item) = buffer.shift().unwrap() {
            result.push(item);
        }

        // Should be in descending order (max-heap)
        let mut expected = numbers.clone();
        expected.sort_by(|a, b| b.cmp(a)); // Sort descending
        assert_eq!(result, expected);
    }

    #[test]
    fn test_interleaved_push_and_shift() {
        let buffer = ExternalBufferQueue::new();

        buffer.push(3).unwrap();
        buffer.push(1).unwrap();
        assert_eq!(buffer.shift().unwrap(), Some(3)); // Max so far

        buffer.push(4).unwrap();
        buffer.push(2).unwrap();
        assert_eq!(buffer.shift().unwrap(), Some(4)); // New max
        assert_eq!(buffer.shift().unwrap(), Some(2));
        assert_eq!(buffer.shift().unwrap(), Some(1));
        assert!(buffer.shift().unwrap().is_none());
    }

    #[test]
    fn test_same_priority_items() {
        let buffer = ExternalBufferQueue::new();

        let item1 = TestItem::new(5, 1, "first");
        let item2 = TestItem::new(5, 2, "second");
        let item3 = TestItem::new(5, 3, "third");

        buffer.push(item1.clone()).unwrap();
        buffer.push(item2.clone()).unwrap();
        buffer.push(item3.clone()).unwrap();

        // All have same priority, but different ids
        // Order should be determined by the secondary field (id)
        let first = buffer.shift().unwrap().unwrap();
        let second = buffer.shift().unwrap().unwrap();
        let third = buffer.shift().unwrap().unwrap();

        assert_eq!(first.priority, 5);
        assert_eq!(second.priority, 5);
        assert_eq!(third.priority, 5);

        // Should maintain heap property
        assert!(first >= second);
        assert!(second >= third);
    }

    #[test]
    fn test_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let buffer = Arc::new(ExternalBufferQueue::new());
        let mut handles = vec![];

        // Spawn multiple threads to push items
        for i in 0..10 {
            let buffer_clone = Arc::clone(&buffer);
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    buffer_clone.push(i * 10 + j).unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all pushes to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Collect all items
        let mut items = Vec::new();
        while let Some(item) = buffer.shift().unwrap() {
            items.push(item);
        }

        // Should have 100 items total
        assert_eq!(items.len(), 100);

        // Should be in descending order
        for window in items.windows(2) {
            assert!(window[0] >= window[1]);
        }
    }

    #[test]
    fn test_large_dataset() {
        let buffer = ExternalBufferQueue::new();

        // Push a large number of items
        let n = 1000;
        for i in 0..n {
            buffer.push(i).unwrap();
        }

        // Verify all items come out in correct order
        for expected in (0..n).rev() {
            assert_eq!(buffer.shift().unwrap(), Some(expected));
        }

        assert!(buffer.shift().unwrap().is_none());
    }

    #[test]
    fn test_error_handling_with_poisoned_mutex() {
        use std::panic;
        use std::sync::Arc;
        use std::thread;

        let buffer = Arc::new(ExternalBufferQueue::new());
        buffer.push(1).unwrap(); // Add an item first

        let buffer_clone = Arc::clone(&buffer);

        // Create a thread that will panic while holding the mutex
        let handle = thread::spawn(move || {
            let _guard = buffer_clone.queue.lock().unwrap();
            panic!("Intentional panic to poison mutex");
        });

        // Wait for the thread to panic and poison the mutex
        assert!(handle.join().is_err());

        // Now trying to use the buffer should return an error
        assert!(buffer.push(2).is_err());
        assert!(buffer.shift().is_err());
    }
}
