// concurrency_config.rs
// Global concurrency configuration for MetaSort
// Copyright (c) 2025 - Sanmith S.
// Portions Copyright (c) 2025 - Berkay.

use std::sync::OnceLock;
use std::io::{self, Write};

/// Concurrency level options for parallel processing
#[derive(Debug, Clone, Copy)]
pub enum ConcurrencyLevel {
    Slow,
    Medium,
    Unlimited,
}

impl ConcurrencyLevel {
    /// Calculate thread count based on available CPU cores
    pub fn get_thread_count(&self) -> usize {
        let num_cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        match self {
            ConcurrencyLevel::Slow => num_cpus.min(4).max(2),        // 2-4 threads
            ConcurrencyLevel::Medium => (num_cpus / 2).max(2),        // Half cores (min 2)
            ConcurrencyLevel::Unlimited => num_cpus,                  // All cores
        }
    }

    /// Parse concurrency level from user input
    fn from_user_input(input: &str) -> Option<Self> {
        match input.trim() {
            "1" => Some(ConcurrencyLevel::Slow),
            "2" => Some(ConcurrencyLevel::Medium),
            "3" => Some(ConcurrencyLevel::Unlimited),
            _ => None,
        }
    }
}

/// Global storage for concurrency level
static CONCURRENCY_LEVEL: OnceLock<ConcurrencyLevel> = OnceLock::new();

/// Initialize concurrency configuration with user prompt
/// This should be called once at application startup
pub fn initialize_concurrency() -> ConcurrencyLevel {
    println!("\nSelect processing speed (concurrency level) for all operations:");
    println!("1. Slow (2-4 threads) - For limited resources");
    println!("2. Medium (half CPU cores) - RECOMMENDED for balanced performance");
    println!("3. Unlimited (all CPU cores) - Maximum speed, high resource usage");
    print!("Enter 1, 2, or 3: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    let level = ConcurrencyLevel::from_user_input(&input)
        .unwrap_or_else(|| {
            println!("[WARNING] Invalid input, defaulting to Medium (recommended)");
            ConcurrencyLevel::Medium
        });

    let thread_count = level.get_thread_count();
    println!("Using {} threads for all parallel operations\n", thread_count);

    // Store in global static
    CONCURRENCY_LEVEL.set(level).ok();
    level
}

/// Get the configured concurrency level
/// Returns Medium as default if not initialized
pub fn get_concurrency_level() -> ConcurrencyLevel {
    *CONCURRENCY_LEVEL.get().unwrap_or(&ConcurrencyLevel::Medium)
}
