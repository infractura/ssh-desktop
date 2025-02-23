use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use tracing::{debug, warn};

const MIN_DISPLAY: u16 = 100;  // Start at :100 to avoid conflicts
const MAX_DISPLAY: u16 = 599;  // Allow up to 500 displays

#[derive(Debug, Clone)]
pub struct DisplayPool {
    used_displays: Arc<Mutex<HashSet<u16>>>,
}

impl DisplayPool {
    pub fn new() -> Self {
        Self {
            used_displays: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Allocate a new display number
    pub async fn allocate(&self) -> Result<u16> {
        let mut displays = self.used_displays.lock().await;
        
        // Find first available display number
        for display in MIN_DISPLAY..=MAX_DISPLAY {
            if !displays.contains(&display) {
                displays.insert(display);
                debug!(display, "Allocated new display number");
                return Ok(display);
            }
        }
        
        anyhow::bail!("No available display numbers")
    }

    /// Release a display number back to the pool
    pub async fn release(&self, display: u16) {
        let mut displays = self.used_displays.lock().await;
        if displays.remove(&display) {
            debug!(display, "Released display number");
        } else {
            warn!(display, "Attempted to release unallocated display");
        }
    }

    /// Get number of currently allocated displays
    pub async fn allocated_count(&self) -> usize {
        self.used_displays.lock().await.len()
    }
}

impl Default for DisplayPool {
    fn default() -> Self {
        Self::new()
    }
}

// Make DisplayPool available globally
lazy_static::lazy_static! {
    pub static ref DISPLAY_POOL: DisplayPool = DisplayPool::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_display_allocation() {
        let pool = DisplayPool::new();
        
        // Allocate display
        let display = pool.allocate().await.unwrap();
        assert!(display >= MIN_DISPLAY);
        assert!(display <= MAX_DISPLAY);
        
        // Verify it's marked as used
        assert_eq!(pool.allocated_count().await, 1);
        
        // Release display
        pool.release(display).await;
        assert_eq!(pool.allocated_count().await, 0);
        
        // Should be able to allocate same number again
        let new_display = pool.allocate().await.unwrap();
        assert_eq!(display, new_display);
    }

    #[tokio::test]
    async fn test_multiple_allocations() {
        let pool = DisplayPool::new();
        let mut displays = Vec::new();
        
        // Allocate 10 displays
        for _ in 0..10 {
            displays.push(pool.allocate().await.unwrap());
        }
        
        assert_eq!(pool.allocated_count().await, 10);
        
        // All numbers should be unique
        let mut unique = HashSet::new();
        displays.iter().for_each(|d| {
            assert!(unique.insert(d));
        });
        
        // Release all
        for display in displays {
            pool.release(display).await;
        }
        
        assert_eq!(pool.allocated_count().await, 0);
    }
}
