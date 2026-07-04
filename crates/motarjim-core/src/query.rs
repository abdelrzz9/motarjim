//! Query system for caching and executing compilation queries.
//!
//! Provides a generic, type-safe query caching framework inspired by
//! incremental computation systems. Queries implement the [`Query`] trait
//! and results are stored in a [`QueryCache`] for reuse across compilation
//! phases.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use dashmap::DashMap;
use motarjim_diag::DiagnosticBag;

/// Internal combined cache key produced from a query type ID and a
/// query-specific key hash.
type QueryKey = u64;

/// A cached entry together with its dependency metadata.
struct CachedValue {
    /// The stored result value as a type-erased heap allocation.
    value: Box<dyn Any + Send>,
    /// File paths this result depends on (used for dependency-based invalidation).
    dependencies: Vec<String>,
    /// Timestamp when this entry was created.
    #[allow(dead_code)]
    created_at: Instant,
}

/// Computes a 64-bit hash for any [`Hash`] value using the default SipHash
/// hasher.
#[must_use]
fn calculate_hash<H: Hash>(value: &H) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Combines a [`TypeId`] and a key hash into a single [`QueryKey`] (u64).
///
/// The type ID is hashed first, then the result is mixed with the key hash
/// using the golden ratio constant `0x9E3779B97F4A7C15` to reduce collision
/// probability.
#[must_use]
fn make_query_key(type_id: TypeId, key_hash: u64) -> QueryKey {
    let tid_hash = calculate_hash(&type_id);
    tid_hash
        .wrapping_mul(0x9E3779B97F4A7C15)
        .wrapping_add(key_hash)
}

/// Specifies the caching behavior and invalidation triggers for a query.
#[derive(Debug, Clone)]
pub enum InvalidationPattern {
    /// Invalidate cached results whenever any file changes.
    OnFileChange,
    /// Invalidate when a file that this result depends on changes.
    OnDependencyChange,
    /// Never cache — always re-execute the query.
    AlwaysExecute,
    /// Invalidate only when one of the explicitly listed files changes.
    OnFileSetChange(Vec<String>),
}

/// Context passed to every query execution.
///
/// Provides access to the shared cache, a cancellation flag, and a
/// diagnostics collector.
pub struct QueryContext {
    /// Reference to the global query cache.
    pub cache: Arc<QueryCache>,
    /// Cancellation flag checked by long-running queries.
    pub cancellation: Arc<AtomicBool>,
    /// Diagnostics collector shared across queries.
    pub diagnostics: Arc<Mutex<DiagnosticBag>>,
}

impl std::fmt::Debug for QueryContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryContext")
            .field("cache", &self.cache)
            .field("cancellation", &self.cancellation)
            .field("diagnostics", &self.diagnostics)
            .finish()
    }
}

/// A composable, cacheable compilation query.
///
/// Implementations define an input key type, an output value type, an
/// execution body, and a caching policy.
pub trait Query: Send + Sync {
    /// The type of the query's input key.
    type Key: Eq + Hash + Clone + Send;
    /// The type of the query's output value.
    type Value: Clone + Send;

    /// Returns a human-readable description of this query.
    fn description(&self) -> &'static str;

    /// Executes the query and produces a value.
    fn execute(&self, key: &Self::Key, context: &QueryContext) -> Self::Value;

    /// Returns the invalidation pattern for this query.
    fn invalidation_pattern(&self) -> InvalidationPattern;
}

/// A concurrent, type-safe cache for query results.
///
/// Results are indexed by query type (via [`TypeId`]) and query key,
/// enabling independent caching for different query types. Access is
/// thread-safe through [`DashMap`].
pub struct QueryCache {
    /// The underlying storage: type ID → (hashed key → cached value).
    results: DashMap<TypeId, HashMap<QueryKey, CachedValue>>,
    /// Number of cache hits since construction.
    hits: AtomicU64,
    /// Number of cache misses since construction.
    misses: AtomicU64,
}

impl std::fmt::Debug for QueryCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryCache")
            .field(
                "results",
                &format_args!("DashMap<{} entries>", self.results.len()),
            )
            .field("hits", &self.hit_count())
            .field("misses", &self.miss_count())
            .finish()
    }
}

impl QueryCache {
    /// Creates a new, empty query cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: DashMap::new(),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// Returns the cached value for `key` in query `Q`, or executes `Q` to
    /// compute and cache it.
    ///
    /// If the query's [`InvalidationPattern`] is `AlwaysExecute`, the cache
    /// is bypassed entirely and the query is always re-executed.
    #[allow(clippy::missing_panics_doc)]
    pub fn get_or_compute<Q: Query + 'static>(
        &self,
        query: &Q,
        key: &Q::Key,
        context: &QueryContext,
    ) -> Q::Value
    where
        Q::Key: Hash,
        Q::Value: 'static,
    {
        if matches!(
            query.invalidation_pattern(),
            InvalidationPattern::AlwaysExecute
        ) {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return query.execute(key, context);
        }

        let type_id = TypeId::of::<Q>();
        let cache_key = make_query_key(type_id, calculate_hash(key));

        if let Some(inner) = self.results.get(&type_id) {
            if let Some(cached) = inner.get(&cache_key) {
                if let Some(value) = cached.value.downcast_ref::<Q::Value>() {
                    self.hits.fetch_add(1, Ordering::Relaxed);
                    return value.clone();
                }
            }
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        let value = query.execute(key, context);

        self.results
            .entry(type_id)
            .or_insert_with(HashMap::new)
            .insert(
                cache_key,
                CachedValue {
                    value: Box::new(value.clone()),
                    dependencies: Vec::new(),
                    created_at: Instant::now(),
                },
            );

        value
    }

    /// Invalidates cached entries matching the given pattern.
    ///
    /// * [`InvalidationPattern::OnFileChange`] — clears all cached results.
    /// * [`InvalidationPattern::AlwaysExecute`] — clears all cached results.
    /// * [`InvalidationPattern::OnDependencyChange`] — removes entries whose
    ///   dependencies overlap with `changed_files`.
    /// * [`InvalidationPattern::OnFileSetChange(files)`] — removes entries
    ///   whose dependencies overlap with the specified file set.
    pub fn invalidate(&self, pattern: &InvalidationPattern, changed_files: &[String]) {
        match pattern {
            InvalidationPattern::OnFileChange | InvalidationPattern::AlwaysExecute => {
                self.results.clear();
            }
            InvalidationPattern::OnDependencyChange => {
                self.results.retain(|_type_id, entries| {
                    entries.retain(|_key, cached| {
                        !cached
                            .dependencies
                            .iter()
                            .any(|dep| changed_files.contains(dep))
                    });
                    !entries.is_empty()
                });
            }
            InvalidationPattern::OnFileSetChange(files) => {
                self.results.retain(|_type_id, entries| {
                    entries.retain(|_key, cached| {
                        !cached.dependencies.iter().any(|dep| files.contains(dep))
                    });
                    !entries.is_empty()
                });
            }
        }
    }

    /// Removes all entries from the cache and resets hit/miss counters.
    pub fn clear(&self) {
        self.results.clear();
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }

    /// Returns the number of cache hits since construction.
    #[must_use]
    pub fn hit_count(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Returns the number of cache misses since construction.
    #[must_use]
    pub fn miss_count(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct LenQuery;

    impl Query for LenQuery {
        type Key = String;
        type Value = usize;

        fn description(&self) -> &'static str {
            "string length query"
        }

        fn execute(&self, key: &String, _context: &QueryContext) -> usize {
            key.len()
        }

        fn invalidation_pattern(&self) -> InvalidationPattern {
            InvalidationPattern::OnFileChange
        }
    }

    struct UppercaseQuery;

    impl Query for UppercaseQuery {
        type Key = String;
        type Value = String;

        fn description(&self) -> &'static str {
            "uppercase query"
        }

        fn execute(&self, key: &String, _context: &QueryContext) -> String {
            key.to_uppercase()
        }

        fn invalidation_pattern(&self) -> InvalidationPattern {
            InvalidationPattern::OnDependencyChange
        }
    }

    struct AlwaysExecQuery;

    impl Query for AlwaysExecQuery {
        type Key = String;
        type Value = String;

        fn description(&self) -> &'static str {
            "always-execute query"
        }

        fn execute(&self, key: &String, _context: &QueryContext) -> String {
            format!("exec:{key}")
        }

        fn invalidation_pattern(&self) -> InvalidationPattern {
            InvalidationPattern::AlwaysExecute
        }
    }

    struct IntKeyQuery;

    impl Query for IntKeyQuery {
        type Key = i32;
        type Value = String;

        fn description(&self) -> &'static str {
            "int key query"
        }

        fn execute(&self, key: &i32, _context: &QueryContext) -> String {
            format!("val:{key}")
        }

        fn invalidation_pattern(&self) -> InvalidationPattern {
            InvalidationPattern::OnFileChange
        }
    }

    fn context(cache: &Arc<QueryCache>) -> QueryContext {
        QueryContext {
            cache: Arc::clone(cache),
            cancellation: Arc::new(AtomicBool::new(false)),
            diagnostics: Arc::new(Mutex::new(DiagnosticBag::new())),
        }
    }

    #[test]
    fn test_basic_get_or_compute() {
        let cache = Arc::new(QueryCache::new());
        let ctx = context(&cache);
        let q = LenQuery;

        let result = cache.get_or_compute(&q, &"hello".to_string(), &ctx);
        assert_eq!(result, 5);
    }

    #[test]
    fn test_caching_returns_same_value() {
        let cache = Arc::new(QueryCache::new());
        let ctx = context(&cache);
        let q = LenQuery;

        let v1 = cache.get_or_compute(&q, &"hello".to_string(), &ctx);
        let v2 = cache.get_or_compute(&q, &"hello".to_string(), &ctx);

        assert_eq!(v1, v2);
        assert_eq!(cache.hit_count(), 1);
        assert_eq!(cache.miss_count(), 1);
    }

    #[test]
    fn test_different_keys_produce_different_cache_entries() {
        let cache = Arc::new(QueryCache::new());
        let ctx = context(&cache);
        let q = LenQuery;

        let v1 = cache.get_or_compute(&q, &"hello".to_string(), &ctx);
        let v2 = cache.get_or_compute(&q, &"world!".to_string(), &ctx);

        assert_eq!(v1, 5);
        assert_eq!(v2, 6);

        let v3 = cache.get_or_compute(&q, &"hello".to_string(), &ctx);
        let v4 = cache.get_or_compute(&q, &"world!".to_string(), &ctx);

        assert_eq!(v3, 5);
        assert_eq!(v4, 6);
        assert_eq!(cache.hit_count(), 2);
        assert_eq!(cache.miss_count(), 2);
    }

    #[test]
    fn test_multiple_query_types_independent() {
        let cache = Arc::new(QueryCache::new());
        let ctx = context(&cache);
        let len_q = LenQuery;
        let upper_q = UppercaseQuery;

        let l1 = cache.get_or_compute(&len_q, &"hello".to_string(), &ctx);
        let u1 = cache.get_or_compute(&upper_q, &"hello".to_string(), &ctx);

        assert_eq!(l1, 5);
        assert_eq!(u1, "HELLO");
        assert_eq!(cache.miss_count(), 2);
        assert_eq!(cache.hit_count(), 0);

        let l2 = cache.get_or_compute(&len_q, &"hello".to_string(), &ctx);
        let u2 = cache.get_or_compute(&upper_q, &"hello".to_string(), &ctx);

        assert_eq!(l2, 5);
        assert_eq!(u2, "HELLO");
        assert_eq!(cache.hit_count(), 2);
        assert_eq!(cache.miss_count(), 2);
    }

    #[test]
    fn test_always_execute_re_executes() {
        let cache = Arc::new(QueryCache::new());
        let ctx = context(&cache);
        let q = AlwaysExecQuery;

        let v1 = cache.get_or_compute(&q, &"key".to_string(), &ctx);
        let v2 = cache.get_or_compute(&q, &"key".to_string(), &ctx);

        assert_eq!(v1, "exec:key");
        assert_eq!(v2, "exec:key");
        // Both should be misses because AlwaysExecute never caches.
        assert_eq!(cache.miss_count(), 2);
        assert_eq!(cache.hit_count(), 0);
    }

    #[test]
    fn test_invalidate_on_file_change_clears_cache() {
        let cache = Arc::new(QueryCache::new());
        let ctx = context(&cache);
        let q = LenQuery;

        cache.get_or_compute(&q, &"hello".to_string(), &ctx);
        assert_eq!(cache.hit_count(), 0);
        assert_eq!(cache.miss_count(), 1);

        cache.invalidate(&InvalidationPattern::OnFileChange, &[]);

        cache.get_or_compute(&q, &"hello".to_string(), &ctx);
        assert_eq!(cache.miss_count(), 2);
    }

    #[test]
    fn test_clear_resets_counters() {
        let cache = Arc::new(QueryCache::new());
        let ctx = context(&cache);
        let q = LenQuery;

        cache.get_or_compute(&q, &"hello".to_string(), &ctx);
        cache.get_or_compute(&q, &"hello".to_string(), &ctx);
        assert_eq!(cache.hit_count(), 1);
        assert_eq!(cache.miss_count(), 1);

        cache.clear();
        assert_eq!(cache.hit_count(), 0);
        assert_eq!(cache.miss_count(), 0);
    }

    #[test]
    fn test_different_key_types() {
        let cache = Arc::new(QueryCache::new());
        let ctx = context(&cache);
        let len_q = LenQuery;
        let int_q = IntKeyQuery;

        let l = cache.get_or_compute(&len_q, &"abc".to_string(), &ctx);
        let i = cache.get_or_compute(&int_q, &42, &ctx);

        assert_eq!(l, 3);
        assert_eq!(i, "val:42");
    }

    #[test]
    fn test_on_file_set_change_invalidation() {
        let cache = Arc::new(QueryCache::new());
        let ctx = context(&cache);
        let q = LenQuery;

        cache.get_or_compute(&q, &"hello".to_string(), &ctx);
        assert_eq!(cache.miss_count(), 1);
        assert_eq!(cache.hit_count(), 0);

        // Invalidate with a file set that doesn't affect this cache
        // (the cached entry has no dependencies, so it won't be removed).
        cache.invalidate(
            &InvalidationPattern::OnFileSetChange(vec!["unrelated.txt".to_string()]),
            &[],
        );

        cache.get_or_compute(&q, &"hello".to_string(), &ctx);
        // The entry has no dependencies, so it should still be cached.
        assert_eq!(cache.hit_count(), 1);
        assert_eq!(cache.miss_count(), 1);
    }
}
