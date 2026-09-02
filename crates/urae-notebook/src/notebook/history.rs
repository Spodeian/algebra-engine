//! # `urae_notebook::notebook::history`
//!
//! Undo / Redo History and Transactional Snapshots.

use crate::notebook::session::{CardDisplayMode, SymbolMetadata};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Individual state snapshot stored in undo/redo history stacks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistorySnapshot {
    pub raw_document_text: String,
    pub slider_values: HashMap<String, f64>,
    pub symbol_metadata: HashMap<String, SymbolMetadata>,
    pub card_modes: HashMap<String, CardDisplayMode>,
    pub description: String,
}

/// Transactional undo / redo history manager for URAE notebook sessions.
#[derive(Debug, Clone)]
pub struct UndoRedoHistory {
    pub undo_stack: VecDeque<HistorySnapshot>,
    pub redo_stack: VecDeque<HistorySnapshot>,
    pub max_history: usize,
}

impl Default for UndoRedoHistory {
    fn default() -> Self {
        Self::new(100)
    }
}

impl UndoRedoHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_history: if max_history == 0 { 100 } else { max_history },
        }
    }

    /// Push a snapshot onto the undo stack and clear redo stack.
    pub fn push(&mut self, snapshot: HistorySnapshot) {
        if let Some(last) = self.undo_stack.back() {
            if last.raw_document_text == snapshot.raw_document_text
                && last.slider_values == snapshot.slider_values
                && last.symbol_metadata == snapshot.symbol_metadata
            {
                return;
            }
        }
        self.undo_stack.push_back(snapshot);
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.pop_front();
        }
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.back().map(|s| s.description.as_str())
    }

    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.back().map(|s| s.description.as_str())
    }
}
