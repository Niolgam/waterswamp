//! Generic pagination types for consistent API responses.
//!
//! This module provides reusable pagination structures to avoid duplication
//! across different model types.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Generic paginated response wrapper.
///
/// This provides a consistent structure for all paginated API responses,
/// reducing code duplication across different entity types.
///
/// # Type Parameters
///
/// * `T` - The type of items in the paginated response
///
/// # Example
///
/// ```rust,ignore
/// use domain::pagination::Paginated;
/// use domain::models::UserDto;
///
/// let response: Paginated<UserDto> = Paginated::new(users, total, limit, offset);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Paginated<T> {
    /// The items in this page
    pub items: Vec<T>,
    /// Total number of items across all pages
    pub total: i64,
    /// Maximum number of items per page
    pub limit: i64,
    /// Number of items skipped (for pagination)
    pub offset: i64,
}

impl<T> Paginated<T> {
    /// Creates a new paginated response.
    pub fn new(items: Vec<T>, total: i64, limit: i64, offset: i64) -> Self {
        Self {
            items,
            total,
            limit,
            offset,
        }
    }

    /// Creates an empty paginated response.
    pub fn empty(limit: i64, offset: i64) -> Self {
        Self {
            items: Vec::new(),
            total: 0,
            limit,
            offset,
        }
    }

    /// Returns true if there are more pages available.
    pub fn has_next(&self) -> bool {
        self.offset + self.limit < self.total
    }

    /// Returns true if this is not the first page.
    pub fn has_previous(&self) -> bool {
        self.offset > 0
    }

    /// Returns the current page number (1-indexed).
    pub fn current_page(&self) -> i64 {
        if self.limit == 0 {
            1
        } else {
            (self.offset / self.limit) + 1
        }
    }

    /// Returns the total number of pages.
    pub fn total_pages(&self) -> i64 {
        if self.limit == 0 {
            1
        } else {
            (self.total + self.limit - 1) / self.limit
        }
    }

    /// Maps the items to a different type.
    pub fn map<U, F>(self, f: F) -> Paginated<U>
    where
        F: FnMut(T) -> U,
    {
        Paginated {
            items: self.items.into_iter().map(f).collect(),
            total: self.total,
            limit: self.limit,
            offset: self.offset,
        }
    }
}

impl<T> Default for Paginated<T> {
    fn default() -> Self {
        Self::empty(10, 0)
    }
}

/// Common query parameters for list operations.
///
/// This provides a consistent structure for pagination and search parameters
/// across different API endpoints.
#[derive(Debug, Clone, Deserialize, ToSchema, Default)]
pub struct ListQuery {
    /// Maximum number of items to return (default: 10)
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Number of items to skip (default: 0)
    #[serde(default)]
    pub offset: i64,
    /// Optional search term
    pub search: Option<String>,
}

fn default_limit() -> i64 {
    10
}

impl ListQuery {
    /// Creates a new list query with the given parameters.
    pub fn new(limit: i64, offset: i64, search: Option<String>) -> Self {
        Self {
            limit,
            offset,
            search,
        }
    }

    /// Returns the effective limit, clamped to reasonable bounds.
    pub fn effective_limit(&self) -> i64 {
        self.limit.clamp(1, 100)
    }

    /// Returns the effective offset, ensuring it's non-negative.
    pub fn effective_offset(&self) -> i64 {
        self.offset.max(0)
    }

    /// Returns the search term if it's not empty.
    pub fn effective_search(&self) -> Option<&str> {
        self.search
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginated_new() {
        let items = vec![1, 2, 3];
        let paginated = Paginated::new(items, 100, 10, 0);

        assert_eq!(paginated.items.len(), 3);
        assert_eq!(paginated.total, 100);
        assert_eq!(paginated.limit, 10);
        assert_eq!(paginated.offset, 0);
    }

    #[test]
    fn test_paginated_has_next() {
        let paginated: Paginated<i32> = Paginated::new(vec![], 100, 10, 0);
        assert!(paginated.has_next());

        let paginated: Paginated<i32> = Paginated::new(vec![], 100, 10, 90);
        assert!(!paginated.has_next());
    }

    #[test]
    fn test_paginated_current_page() {
        let paginated: Paginated<i32> = Paginated::new(vec![], 100, 10, 0);
        assert_eq!(paginated.current_page(), 1);

        let paginated: Paginated<i32> = Paginated::new(vec![], 100, 10, 20);
        assert_eq!(paginated.current_page(), 3);
    }

    #[test]
    fn test_paginated_total_pages() {
        let paginated: Paginated<i32> = Paginated::new(vec![], 100, 10, 0);
        assert_eq!(paginated.total_pages(), 10);

        let paginated: Paginated<i32> = Paginated::new(vec![], 95, 10, 0);
        assert_eq!(paginated.total_pages(), 10);
    }

    #[test]
    fn test_paginated_map() {
        let paginated = Paginated::new(vec![1, 2, 3], 3, 10, 0);
        let mapped = paginated.map(|x| x * 2);

        assert_eq!(mapped.items, vec![2, 4, 6]);
        assert_eq!(mapped.total, 3);
    }

    #[test]
    fn test_list_query_effective_limit() {
        let query = ListQuery::new(1000, 0, None);
        assert_eq!(query.effective_limit(), 100);

        let query = ListQuery::new(-5, 0, None);
        assert_eq!(query.effective_limit(), 1);
    }

    #[test]
    fn test_list_query_effective_search() {
        let query = ListQuery::new(10, 0, Some("  test  ".to_string()));
        assert_eq!(query.effective_search(), Some("test"));

        let query = ListQuery::new(10, 0, Some("   ".to_string()));
        assert_eq!(query.effective_search(), None);
    }
}
