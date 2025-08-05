pub fn spawn(fut: impl futures::Future<Output = ()> + Send + 'static) {
    #[cfg(feature = "rt-tokio")]
    {
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::spawn(fut);
            return;
        }
    }

    std::thread::spawn(move || {
        futures::executor::block_on(fut);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn test_spawn_without_tokio_runtime() {
        // 测试在没有 Tokio runtime 的情况下使用 spawn
        let executed = Arc::new(Mutex::new(false));
        let executed_clone = executed.clone();

        spawn(async move {
            *executed_clone.lock().unwrap() = true;
        });

        // 等待一下让任务执行完成
        std::thread::sleep(Duration::from_millis(100));

        assert!(*executed.lock().unwrap(), "Task should have been executed");
    }

    #[cfg(feature = "rt-tokio")]
    #[tokio::test]
    async fn test_spawn_with_tokio_runtime() {
        // 测试在 Tokio runtime 中使用 spawn
        let executed = Arc::new(Mutex::new(false));
        let executed_clone = executed.clone();

        spawn(async move {
            *executed_clone.lock().unwrap() = true;
        });

        // 等待一下让任务执行完成
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(
            *executed.lock().unwrap(),
            "Task should have been executed in tokio runtime"
        );
    }

    #[cfg(feature = "rt-tokio")]
    #[test]
    fn test_spawn_fallback_when_no_tokio_context() {
        // 测试当 rt-tokio feature 启用但没有 Tokio runtime 上下文时的回退行为
        // 这个测试在非 Tokio 上下文中运行

        let executed = Arc::new(Mutex::new(false));
        let executed_clone = executed.clone();

        // 确保我们不在 Tokio runtime 中
        assert!(
            tokio::runtime::Handle::try_current().is_err(),
            "This test should run outside tokio runtime"
        );

        spawn(async move {
            *executed_clone.lock().unwrap() = true;
        });

        // 等待一下让任务执行完成
        std::thread::sleep(Duration::from_millis(100));

        assert!(
            *executed.lock().unwrap(),
            "Task should have been executed via thread fallback"
        );
    }

    #[test]
    fn test_spawn_multiple_tasks() {
        // 测试同时启动多个任务
        let counter = Arc::new(Mutex::new(0));
        let num_tasks = 5;

        for i in 0..num_tasks {
            let counter_clone = counter.clone();
            spawn(async move {
                // 模拟一些异步工作，使用标准库的 sleep 而不是 tokio::time::sleep
                std::thread::sleep(Duration::from_millis(10 * (i + 1) as u64));
                *counter_clone.lock().unwrap() += 1;
            });
        }

        // 等待所有任务完成
        std::thread::sleep(Duration::from_millis(200));

        assert_eq!(
            *counter.lock().unwrap(),
            num_tasks,
            "All {} tasks should have been executed",
            num_tasks
        );
    }

    #[test]
    fn test_spawn_with_panic_handling() {
        // 测试任务中的 panic 不会影响主线程
        let executed_after_panic = Arc::new(Mutex::new(false));
        let executed_clone = executed_after_panic.clone();

        // 启动一个会 panic 的任务
        spawn(async {
            panic!("This should not crash the main thread");
        });

        // 启动另一个正常的任务
        spawn(async move {
            std::thread::sleep(Duration::from_millis(50));
            *executed_clone.lock().unwrap() = true;
        });

        // 等待一下
        std::thread::sleep(Duration::from_millis(100));

        assert!(
            *executed_after_panic.lock().unwrap(),
            "Normal task should execute even after another task panics"
        );
    }
}
