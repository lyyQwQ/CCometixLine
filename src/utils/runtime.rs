use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Global tokio runtime singleton for async operations
/// This avoids creating a new runtime on every statusline call
pub static GLOBAL_RUNTIME: Lazy<Arc<Runtime>> =
    Lazy::new(|| Arc::new(Runtime::new().expect("Failed to create tokio runtime")));

/// Execute an async function using the global runtime
pub fn block_on<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    GLOBAL_RUNTIME.block_on(future)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_runtime() {
        // Test that the runtime can execute async tasks
        let result = block_on(async {
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            42
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_runtime_singleton() {
        // Verify that multiple calls use the same runtime instance
        let runtime1 = GLOBAL_RUNTIME.clone();
        let runtime2 = GLOBAL_RUNTIME.clone();
        assert!(Arc::ptr_eq(&runtime1, &runtime2));
    }
}
