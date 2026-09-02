//! Thread-safe symbol table interner for variable and function names.

use crate::id::SymbolId;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;

/// Lock-free, thread-safe symbol interner mapping string names to compact [`SymbolId`] handles.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    lookup: Arc<DashMap<String, SymbolId>>,
    names: Arc<RwLock<Vec<String>>>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    /// Create a new empty symbol table.
    pub fn new() -> Self {
        Self {
            lookup: Arc::new(DashMap::new()),
            names: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Intern a string symbol name and return its unique [`SymbolId`].
    ///
    /// If the name is already interned, returns the existing [`SymbolId`].
    pub fn get_or_intern<S: AsRef<str>>(&self, name: S) -> SymbolId {
        let name_ref = name.as_ref();
        if let Some(id) = self.lookup.get(name_ref) {
            return *id;
        }

        let mut names_guard = self.names.write();
        // Double-check pattern after acquiring write lock
        if let Some(id) = self.lookup.get(name_ref) {
            return *id;
        }

        let new_id = SymbolId::new(names_guard.len() as u32);
        let name_string = name_ref.to_string();
        names_guard.push(name_string.clone());
        self.lookup.insert(name_string, new_id);

        new_id
    }

    /// Retrieve the string name associated with a [`SymbolId`].
    pub fn resolve(&self, id: SymbolId) -> Option<String> {
        let names_guard = self.names.read();
        names_guard.get(id.as_usize()).cloned()
    }

    /// Get the [`SymbolId`] for a symbol name if already interned.
    pub fn get<S: AsRef<str>>(&self, name: S) -> Option<SymbolId> {
        self.lookup.get(name.as_ref()).map(|r| *r)
    }

    /// Check if a symbol name is currently interned.
    pub fn contains<S: AsRef<str>>(&self, name: S) -> bool {
        self.lookup.contains_key(name.as_ref())
    }

    /// Get the total number of interned symbols.
    pub fn len(&self) -> usize {
        self.names.read().len()
    }

    /// Returns `true` if no symbols have been interned.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
