use once_cell::sync::Lazy;

/// Global debug mode flag, initialized once at startup
pub static DEBUG_MODE: Lazy<bool> = Lazy::new(|| std::env::var("CCLINE_DEBUG").is_ok());

/// Conditional debug output macro
///
/// This macro only prints to stderr when DEBUG_MODE is enabled.
/// It avoids the performance overhead of checking environment variables on every call.
///
/// # Examples
///
/// ```
/// debug_println!("Thread pool configuration:");
/// debug_println!("  Physical cores: {}", physical_cores);
/// ```
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        if *$crate::utils::debug::DEBUG_MODE {
            eprintln!($($arg)*);
        }
    };
}

/// Re-export for internal use
pub use debug_println;
