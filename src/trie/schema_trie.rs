// Copyright 2025 Stoolap Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Schema trie implementation for storing table schemas in a Merkle trie
//!
//! This module provides a Merkle trie structure for storing and verifying
//! database table schemas with cryptographic proofs.

use crate::core::{DataType, Schema, SchemaColumn};
use std::collections::HashMap;

/// Column definition for a table
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDef {
    /// Column name
    pub name: String,
    /// Column data type
    pub data_type: DataType,
    /// Whether the column is nullable
    pub nullable: bool,
    /// Whether the column is a primary key
    pub is_primary_key: bool,
}

impl ColumnDef {
    /// Create a new column definition
    pub fn new(name: String, data_type: DataType) -> Self {
        Self {
            name,
            data_type,
            nullable: true,
            is_primary_key: false,
        }
    }

    /// Set whether the column is nullable
    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    /// Set whether the column is a primary key
    pub fn with_primary_key(mut self, is_primary_key: bool) -> Self {
        self.is_primary_key = is_primary_key;
        self
    }
}

impl From<&SchemaColumn> for ColumnDef {
    fn from(col: &SchemaColumn) -> Self {
        Self {
            name: col.name.clone(),
            data_type: col.data_type,
            nullable: col.nullable,
            is_primary_key: col.primary_key,
        }
    }
}

impl From<ColumnDef> for SchemaColumn {
    fn from(def: ColumnDef) -> Self {
        Self {
            id: 0, // Will be set by schema builder
            name: def.name.clone(),
            name_lower: def.name.to_lowercase(),
            data_type: def.data_type,
            nullable: def.nullable,
            primary_key: def.is_primary_key,
            auto_increment: false,
            default_expr: None,
            default_value: None,
            check_expr: None,
            vector_dimensions: 0,
        }
    }
}

/// Table schema definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableSchema {
    /// Table name
    pub name: String,
    /// Column definitions
    pub columns: Vec<ColumnDef>,
}

impl TableSchema {
    /// Create a new table schema
    pub fn new(name: String) -> Self {
        Self {
            name,
            columns: vec![],
        }
    }

    /// Add a column to the schema
    pub fn add_column(&mut self, column: ColumnDef) {
        self.columns.push(column);
    }

    /// Get the hash of the schema
    pub fn hash(&self) -> [u8; 32] {
        // TODO: Implement proper hashing
        // For now, return a simple hash based on name length
        let mut hash = [0u8; 32];
        hash[0] = self.name.len() as u8;
        hash
    }
}

impl From<&Schema> for TableSchema {
    fn from(schema: &Schema) -> Self {
        Self {
            name: schema.table_name.clone(),
            columns: schema.columns.iter().map(ColumnDef::from).collect(),
        }
    }
}

/// A Merkle trie for storing table schemas
///
/// The SchemaTrie provides efficient storage and verification of
/// table schemas with Merkle proofs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaTrie {
    /// Map of table name to schema
    schemas: HashMap<String, TableSchema>,
    /// Root hash of the trie
    root_hash: [u8; 32],
}

impl SchemaTrie {
    /// Create a new empty schema trie
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            root_hash: [0u8; 32],
        }
    }

    /// Insert a table schema into the trie
    pub fn insert(&mut self, schema: TableSchema) {
        let name = schema.name.clone();
        self.schemas.insert(name, schema);
        self.update_root_hash();
    }

    /// Get a table schema from the trie
    pub fn get(&self, table_name: &str) -> Option<&TableSchema> {
        self.schemas.get(table_name)
    }

    /// Get the Merkle root of the trie
    pub fn root_hash(&self) -> [u8; 32] {
        self.root_hash
    }

    /// Update the root hash based on current schemas
    fn update_root_hash(&mut self) {
        // TODO: Implement proper Merkle root computation
        // For now, use a simple hash based on number of schemas
        let mut hash = [0u8; 32];
        hash[0] = self.schemas.len() as u8;
        self.root_hash = hash;
    }

    /// Generate a proof for a table schema
    pub fn generate_proof(&self, _table_name: &str) -> Option<crate::trie::proof::MerkleProof> {
        // TODO: Implement proof generation
        None
    }
}

impl Default for SchemaTrie {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_def_new() {
        let col = ColumnDef::new("test_col".to_string(), DataType::Integer);
        assert_eq!(col.name, "test_col");
        assert_eq!(col.data_type, DataType::Integer);
        assert!(col.nullable);
        assert!(!col.is_primary_key);
    }

    #[test]
    fn test_column_def_builder() {
        let col = ColumnDef::new("id".to_string(), DataType::Integer)
            .with_nullable(false)
            .with_primary_key(true);

        assert!(!col.nullable);
        assert!(col.is_primary_key);
    }

    #[test]
    fn test_table_schema_new() {
        let schema = TableSchema::new("users".to_string());
        assert_eq!(schema.name, "users");
        assert!(schema.columns.is_empty());
    }

    #[test]
    fn test_table_schema_add_column() {
        let mut schema = TableSchema::new("users".to_string());
        schema.add_column(ColumnDef::new("id".to_string(), DataType::Integer));
        schema.add_column(ColumnDef::new("name".to_string(), DataType::Text));

        assert_eq!(schema.columns.len(), 2);
        assert_eq!(schema.columns[0].name, "id");
        assert_eq!(schema.columns[1].name, "name");
    }

    #[test]
    fn test_schema_trie_new() {
        let trie = SchemaTrie::new();
        assert_eq!(trie.root_hash(), [0u8; 32]);
        assert!(trie.get("nonexistent").is_none());
    }

    #[test]
    fn test_schema_trie_insert_and_get() {
        let mut trie = SchemaTrie::new();
        let schema = TableSchema::new("test_table".to_string());
        trie.insert(schema);

        let retrieved = trie.get("test_table");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_table");
    }

    #[test]
    fn test_schema_trie_default() {
        let trie = SchemaTrie::default();
        assert!(trie.schemas.is_empty());
        assert_eq!(trie.root_hash(), [0u8; 32]);
    }
}
