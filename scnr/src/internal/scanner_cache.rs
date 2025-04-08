//! Module with the global scanner cache.
//!
//! The scanner cache is a global cache that stores `ScannerImpl`` instances. It is used to
//! create scanners with the same scanner modes without having to recompile the regular
//! expressions for the patterns and than generate the NFA for the scanner.
//!
//! The scanner cache is a singleton and can be accessed with the `SCANNER_CACHE` constant.
//!
//! # Implementation
//! The global scanner cache is implemented as a `RwLock<ScannerCache>`.
//!

use crate::{scanner_mode::ScannerMode, Result};
use rustc_hash::FxHashMap;

use std::sync::Arc;

use super::ScannerImpl;

/// The cache is a `FxHashMap` that maps a vectors of `ScannerMode` to `Arc<ScannerImpl>`.
pub(crate) struct ScannerCache {
    cache: FxHashMap<Vec<ScannerMode>, Arc<ScannerImpl>>,
}

impl ScannerCache {
    /// Creates a new scanner cache.
    pub(crate) fn new() -> Self {
        Self {
            cache: FxHashMap::default(),
        }
    }

    /// Returns a scanner from the cache or creates a new one if it does not exist.
    ///
    /// # Safety
    /// This function uses `unsafe` because it dereferences a raw pointer.
    /// The Arc assures that the pointer is valid because it holds a reference to the object as
    /// long as the Arc is in the cache and thus alive.
    pub(crate) fn get(&mut self, modes: &[ScannerMode]) -> Result<ScannerImpl> {
        if let Some(scanner) = self.cache.get(modes) {
            // We need to clone the scanner because we need to return a new instance of the scanner.
            // This is because the scanner is mutable and we need to have a unique instance of the
            // scanner.
            // But cloning is much cheaper than creating a new scanner.
            let cloned_scanner =
                unsafe { (*std::sync::Arc::<ScannerImpl>::as_ptr(scanner)).clone() };
            Ok(cloned_scanner)
        } else {
            self.cache
                .insert(modes.to_vec(), Arc::new(modes.try_into()?));
            Ok(self.get(modes).unwrap())
        }
    }
}

/// The global scanner cache.
/// This is a singleton that can be accessed from anywhere in the code.
/// It is a `RwLock` to allow multiple threads to access the cache.
pub(crate) static SCANNER_CACHE: std::sync::LazyLock<std::sync::RwLock<ScannerCache>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(ScannerCache::new()));
