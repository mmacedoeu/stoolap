# HexaryProof Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace binary `MerkleProof` with compact `HexaryProof` for 16-way trie verification

**Architecture:** Replace the binary-tree proof format with a hexary-aware format using bitmap-based sibling encoding, nibble-packed paths, and extension flattening. Uses SHA-256 for cryptographic hashing with streaming verification for security.

**Tech Stack:** Rust, sha2 crate for SHA-256, rayon for parallel verification

---

## Task 1: Add Core HexaryProof Data Structures

**Files:**
- Modify: `src/trie/proof.rs`

**Step 1: Write failing test for HexaryProof basic structure**

```rust
#[test]
fn test_hexary_proof_basic_structure() {
    use crate::trie::proof::HexaryProof;
    use crate::trie::proof::ProofLevel;

    let proof = HexaryProof {
        value_hash: [1u8; 32],
        levels: vec![
            ProofLevel {
                bitmap: 0b1000000000001000,
                siblings: vec![[2u8; 32]],
            }
        ],
        root: [3u8; 32],
        path: vec![0x35], // nibbles [5]
    };

    assert_eq!(proof.value_hash, [1u8; 32]);
    assert_eq!(proof.levels.len(), 1);
    assert_eq!(proof.levels[0].bitmap, 0b1000000000001000);
    assert_eq!(proof.path, vec![0x35]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_hexary_proof_basic_structure --lib`
Expected: FAIL with "cannot find HexaryProof in this crate"

**Step 3: Add HexaryProof and ProofLevel types to src/trie/proof.rs**

Add after line 42 (after MerkleProof struct):

```rust
/// A single level in a hexary Merkle proof
///
/// Contains the sibling information needed to verify one level
/// of a 16-way hexary trie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofLevel {
    /// 16-bit bitmap indicating which child positions have hashes
    /// Bit i is set if child at position i (0-15) exists
    pub bitmap: u16,

    /// Sibling hashes (non-path children only)
    /// Order corresponds to set bits in bitmap (excluding path bit)
    pub siblings: Vec<[u8; 32]>,
}

/// Hexary Merkle proof for 16-way trie verification
///
/// This proof type is designed for hexary (16-way branching) tries like
/// RowTrie, providing compact proofs and efficient verification.
///
/// # Fields
///
/// * `value_hash` - Hash of the value being proven (row hash)
/// * `levels` - Proof levels from root to leaf
/// * `root` - Expected Merkle root
/// * `path` - Nibble path (2 nibbles packed per byte, LSB first)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexaryProof {
    /// Hash of the value being proven
    pub value_hash: [u8; 32],

    /// Proof levels from root to leaf
    pub levels: Vec<ProofLevel>,

    /// Expected Merkle root
    pub root: [u8; 32],

    /// Nibble path (2 nibbles packed per byte, LSB first)
    /// If path length is odd, last byte uses only low nibble
    pub path: Vec<u8>,
}

impl HexaryProof {
    /// Create a new empty hexary proof
    pub fn new() -> Self {
        Self {
            value_hash: [0u8; 32],
            levels: Vec::new(),
            root: [0u8; 32],
            path: Vec::new(),
        }
    }

    /// Create a proof with a value hash
    pub fn with_value_hash(value_hash: [u8; 32]) -> Self {
        Self {
            value_hash,
            levels: Vec::new(),
            root: [0u8; 32],
            path: Vec::new(),
        }
    }

    /// Add a proof level
    pub fn add_level(&mut self, bitmap: u16, siblings: Vec<[u8; 32]>) {
        self.levels.push(ProofLevel { bitmap, siblings });
    }

    /// Set the root hash
    pub fn set_root(&mut self, root: [u8; 32]) {
        self.root = root;
    }

    /// Set the path
    pub fn set_path(&mut self, path: Vec<u8>) {
        self.path = path;
    }
}

impl Default for HexaryProof {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_hexary_proof_basic_structure --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/trie/proof.rs
git commit -m "feat(trie): add HexaryProof and ProofLevel data structures

- Add ProofLevel for hexary trie proof levels
- Add HexaryProof with value_hash, levels, root, path fields
- Implement builder methods for proof construction
- Add basic structure test"
```

---

## Task 2: Add Nibble Packing/Unpacking Utilities

**Files:**
- Modify: `src/trie/proof.rs`

**Step 1: Write failing test for nibble packing**

```rust
#[test]
fn test_pack_nibbles() {
    use crate::trie::proof::pack_nibbles;

    // Even length: [5, 12] -> [0x35, 0xC3]
    let result = pack_nibbles(&[5, 12]);
    assert_eq!(result, vec![0x35, 0xC3]);

    // Odd length: [5, 12, 3] -> [0x35, 0xC3, 0x03]
    let result = pack_nibbles(&[5, 12, 3]);
    assert_eq!(result, vec![0x35, 0xC3, 0x03]);
}

#[test]
fn test_unpack_nibbles() {
    use crate::trie::proof::unpack_nibbles;

    // Even: [0x35, 0xC3] -> [5, 12]
    let result = unpack_nibbles(&[0x35, 0xC3]);
    assert_eq!(result, vec![5, 12]);

    // Odd: [0x35, 0xC3, 0x03] -> [5, 12, 3]
    let result = unpack_nibbles(&[0x35, 0xC3, 0x03]);
    assert_eq!(result, vec![5, 12, 3]);
}

#[test]
fn test_nibble_roundtrip() {
    use crate::trie::proof::{pack_nibbles, unpack_nibbles};

    let original = vec![1, 5, 12, 15, 7, 3];
    let packed = pack_nibbles(&original);
    let unpacked = unpack_nibbles(&packed);
    assert_eq!(original, unpacked);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_pack_nibbles test_unpack_nibbles test_nibble_roundtrip --lib`
Expected: FAIL with "cannot find function"

**Step 3: Implement nibble packing/unpacking functions**

Add to src/trie/proof.rs after the hash_pair function:

```rust
/// Pack nibbles into bytes (2 nibbles per byte, LSB first)
///
/// Each byte contains two nibbles: low nibble first, then high nibble.
/// If the input has odd length, the final byte has the nibble in the low position.
///
/// # Arguments
///
/// * `nibbles` - Slice of nibbles (each 0-15)
///
/// # Returns
///
/// Packed bytes
///
/// # Examples
///
/// ```
/// use stoolap::trie::proof::pack_nibbles;
///
/// let packed = pack_nibbles(&[5, 12]);
/// assert_eq!(packed, vec![0x35]); // 5 in low nibble, 12 (0xC) in high
/// ```
pub fn pack_nibbles(nibbles: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity((nibbles.len() + 1) / 2);

    for chunk in nibbles.chunks(2) {
        let low = chunk[0] & 0x0F;
        let high = if chunk.len() > 1 {
            (chunk[1] & 0x0F) << 4
        } else {
            0
        };
        result.push(low | high);
    }

    result
}

/// Unpack bytes into nibbles (2 nibbles per byte, LSB first)
///
/// # Arguments
///
/// * `packed` - Packed bytes
///
/// # Returns
///
/// Unpacked nibbles
///
/// # Examples
///
/// ```
/// use stoolap::trie::proof::unpack_nibbles;
///
/// let nibbles = unpack_nibbles(&[0x35]);
/// assert_eq!(nibbles, vec![5, 12]);
/// ```
pub fn unpack_nibbles(packed: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(packed.len() * 2);

    for &byte in packed {
        result.push(byte & 0x0F); // Low nibble
        result.push((byte >> 4) & 0x0F); // High nibble
    }

    // Remove trailing zero nibbles from odd-length packing
    while result.last() == Some(&0) && result.len() % 2 == 0 {
        result.pop();
    }

    result
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_pack_nibbles test_unpack_nibbles test_nibble_roundtrip --lib`
Expected: All PASS

**Step 5: Commit**

```bash
git add src/trie/proof.rs
git commit -m "feat(trie): add nibble packing/unpacking utilities

- Add pack_nibbles: 2 nibbles per byte, LSB first
- Add unpack_nibbles: reverse operation
- Handle odd-length paths correctly
- Add roundtrip tests"
```

---

## Task 3: Add Bitmap Sibling Reconstruction

**Files:**
- Modify: `src/trie/proof.rs`

**Step 1: Write failing test for sibling reconstruction**

```rust
#[test]
fn test_reconstruct_children() {
    use crate::trie::proof::reconstruct_children;

    // At level with path nibble 5, siblings at positions 3 and 12
    let bitmap = 0b1000000000001000u16; // bits 3, 5, 12 set
    let siblings = vec![[3u8; 32], [12u8; 32]];
    let path_nibble = 5;
    let our_hash = [5u8; 32];

    let children = reconstruct_children(bitmap, &siblings, path_nibble, our_hash);

    assert_eq!(children[3], [3u8; 32]);
    assert_eq!(children[5], [5u8; 32]); // Our hash
    assert_eq!(children[12], [12u8; 32]);
    assert_eq!(children[0], [0u8; 32]); // Empty
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_reconstruct_children --lib`
Expected: FAIL with "cannot find function"

**Step 3: Implement reconstruct_children function**

Add to src/trie/proof.rs:

```rust
/// Reconstruct the 16-child array from bitmap, siblings, and path
///
/// Given the bitmap of which children exist and the sibling hashes,
/// reconstruct the full 16-element child array with our hash at the path position.
///
/// # Arguments
///
/// * `bitmap` - 16-bit bitmap of existing children
/// * `siblings` - Sibling hashes (non-path children)
/// * `path_nibble` - Which child position we took (0-15)
/// * `our_hash` - Our hash to place at path_nibble position
///
/// # Returns
///
/// Array of 16 child hashes (empty positions are [0; 32])
pub fn reconstruct_children(
    bitmap: u16,
    siblings: &[[u8; 32]],
    path_nibble: u8,
    our_hash: [u8; 32],
) -> [[u8; 32]; 16] {
    let mut children = [[0u8; 32]; 16];
    let mut sibling_idx = 0;

    for i in 0..16 {
        if bitmap & (1 << i) != 0 {
            if i == path_nibble as usize {
                children[i] = our_hash;
            } else {
                if sibling_idx < siblings.len() {
                    children[i] = siblings[sibling_idx];
                    sibling_idx += 1;
                }
            }
        }
    }

    children
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_reconstruct_children --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/trie/proof.rs
git commit -m "feat(trie): add reconstruct_children for hexary proof verification

- Reconstruct 16-child array from bitmap and siblings
- Place our hash at path position
- Return full array for hashing"
```

---

## Task 4: Add 16-Way Child Hashing

**Files:**
- Modify: `src/trie/proof.rs`

**Step 1: Write failing test for 16-way hashing**

```rust
#[test]
fn test_hash_16_children() {
    use crate::trie::proof::hash_16_children;

    let children = [
        [1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32],
        [5u8; 32], [6u8; 32], [7u8; 32], [8u8; 32],
        [9u8; 32], [10u8; 32], [11u8; 32], [12u8; 32],
        [13u8; 32], [14u8; 32], [15u8; 32], [16u8; 32],
    ];

    let result = hash_16_children(&children);
    assert_ne!(result, [0u8; 32]);
    assert_ne!(result, children[0]);

    // Deterministic: same input produces same output
    let result2 = hash_16_children(&children);
    assert_eq!(result, result2);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_hash_16_children --lib`
Expected: FAIL with "cannot find function"

**Step 3: Implement hash_16_children function**

Add to src/trie/proof.rs:

```rust
/// Hash 16 child values using SHA-256
///
/// Concatenates all 16 child hashes and computes SHA-256.
/// This is used to compute the parent hash from child hashes in a hexary trie.
///
/// # Arguments
///
/// * `children` - Array of 16 child hashes
///
/// # Returns
///
/// SHA-256 hash of concatenated children
///
/// # Examples
///
/// ```
/// use stoolap::trie::proof::hash_16_children;
///
/// let children = [[1u8; 32]; 16];
/// let hash = hash_16_children(&children);
/// assert_ne!(hash, [0u8; 32]);
/// ```
pub fn hash_16_children(children: &[[u8; 32]; 16]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for child in children {
        hasher.update(child);
    }
    hasher.finalize().into()
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_hash_16_children --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/trie/proof.rs
git commit -m "feat(trie): add hash_16_children for hexary parent hash computation

- SHA-256 of concatenated 16 child hashes
- Used for computing parent hash in hexary trie"
```

---

## Task 5: Implement HexaryProof Verification

**Files:**
- Modify: `src/trie/proof.rs`

**Step 1: Write failing test for proof verification**

```rust
#[test]
fn test_hexary_proof_verify_valid() {
    use crate::trie::proof::{HexaryProof, ProofLevel, hash_16_children, reconstruct_children};

    // Build a simple proof: value -> level 0 -> root
    let value_hash = [1u8; 32];

    // At level 0: we're at position 5, sibling at position 3
    let level0_bitmap = 0b1010000000001000u16; // bits 3 and 5 set
    let level0_siblings = vec![[3u8; 32]]; // sibling at position 3
    let level0 = ProofLevel {
        bitmap: level0_bitmap,
        siblings: level0_siblings,
    };

    // Reconstruct what the root should be
    let children = reconstruct_children(level0_bitmap, &level0_siblings, 5, value_hash);
    let expected_root = hash_16_children(&children);

    let mut proof = HexaryProof {
        value_hash,
        levels: vec![level0],
        root: expected_root,
        path: vec![0x05], // path nibble 5
    };

    assert!(proof.verify());
}

#[test]
fn test_hexary_proof_verify_invalid_root() {
    use crate::trie::proof::{HexaryProof, ProofLevel};

    let proof = HexaryProof {
        value_hash: [1u8; 32],
        levels: vec![ProofLevel {
            bitmap: 0b1010000000001000,
            siblings: vec![[3u8; 32]],
        }],
        root: [99u8; 32], // Wrong root
        path: vec![0x05],
    };

    assert!(!proof.verify());
}

#[test]
fn test_hexary_proof_verify_path_depth_mismatch() {
    use crate::trie::proof::{HexaryProof, ProofLevel};

    let proof = HexaryProof {
        value_hash: [1u8; 32],
        levels: vec![ProofLevel {
            bitmap: 0b1010000000001000,
            siblings: vec![[3u8; 32]],
        }],
        root: [0u8; 32],
        path: vec![0x05, 0x03], // 2 nibbles but only 1 level
    };

    assert!(!proof.verify());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_hexary_proof_verify --lib`
Expected: FAIL with "no method named verify"

**Step 3: Implement HexaryProof::verify method**

Add to impl HexaryProof block:

```rust
    /// Verify the hexary Merkle proof
    ///
    /// Uses streaming verification from leaf to root, failing immediately
    /// on any mismatch.
    ///
    /// # Returns
    ///
    /// `true` if the proof is valid, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use stoolap::trie::proof::HexaryProof;
    ///
    /// let proof = HexaryProof::new();
    /// // ... populate proof ...
    /// if proof.verify() {
    ///     println!("Proof is valid!");
    /// }
    /// ```
    pub fn verify(&self) -> bool {
        // Start with the value hash
        let mut current_hash = self.value_hash;

        // Unpack path nibbles
        let path_nibbles = unpack_nibbles(&self.path);

        // Path depth must match number of levels
        if path_nibbles.len() != self.levels.len() {
            return false;
        }

        // Verify each level from leaf to root
        for (level_idx, level) in self.levels.iter().enumerate() {
            let path_nibble = path_nibbles[level_idx];

            // Reconstruct the full set of children at this level
            let children = reconstruct_children(level.bitmap, &level.siblings, path_nibble, current_hash);

            // Hash all 16 children to get parent hash
            current_hash = hash_16_children(&children);
        }

        // Final hash should match expected root
        current_hash == self.root
    }
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_hexary_proof_verify --lib`
Expected: All PASS

**Step 5: Commit**

```bash
git add src/trie/proof.rs
git commit -m "feat(trie): implement HexaryProof::verify with streaming verification

- Streaming verification from leaf to root
- Path depth validation
- Reconstruct children at each level
- SHA-256 hash all 16 children
- Fails immediately on mismatch"
```

---

## Task 6: Implement get_hexary_proof in RowTrie

**Files:**
- Modify: `src/trie/row_trie.rs`

**Step 1: Write failing test for proof generation**

```rust
#[test]
fn test_row_trie_get_hexary_proof_basic() {
    use crate::determ::{DetermRow, DetermValue};
    use crate::trie::row_trie::RowTrie;

    let mut trie = RowTrie::new();

    let row1 = DetermRow::from_values(vec![
        DetermValue::integer(1),
        DetermValue::text("Alice"),
    ]);

    trie.insert(1, row1);

    let proof = trie.get_hexary_proof(1);
    assert!(proof.is_some());

    let proof = proof.unwrap();
    assert_eq!(proof.root, trie.get_root());
    assert!(proof.verify());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_row_trie_get_hexary_proof_basic --lib`
Expected: FAIL with "no method named get_hexary_proof"

**Step 3: Implement get_hexary_proof method**

Add to src/trie/row_trie.rs in impl RowTrie block (after get_proof method):

First add the use statement at top of file:
```rust
use crate::trie::proof::{HexaryProof, ProofLevel};
```

Then add the method:

```rust
    /// Generate a hexary Merkle proof for a row
    ///
    /// Returns None if the row doesn't exist.
    pub fn get_hexary_proof(&self, row_id: i64) -> Option<HexaryProof> {
        let key = encode_row_id(row_id);
        let mut levels = Vec::new();
        let mut path = Vec::new();

        let value_hash = self.do_generate_hexary_proof(
            self.root.as_ref().map(|r| r.as_ref()),
            &key,
            0,
            &mut levels,
            &mut path,
            row_id,
        )?;

        Some(HexaryProof {
            value_hash,
            levels,
            root: self.get_root(),
            path,
        })
    }

    /// Helper: pack a nibble into the path (2 nibbles per byte)
    fn pack_nibble(&self, path: &mut Vec<u8>, nibble: u8) {
        if path.len() % 2 == 0 {
            path.push(nibble); // Low nibble of new byte
        } else {
            *path.last_mut().unwrap() |= nibble << 4; // High nibble
        }
    }

    /// Recursive helper for generating hexary proof
    fn do_generate_hexary_proof(
        &self,
        node: Option<&RowNode>,
        key: &[u8],
        depth: usize,
        levels: &mut Vec<ProofLevel>,
        path: &mut Vec<u8>,
        target_row_id: i64,
    ) -> Option<[u8; 32]> {
        match node {
            None => None,
            Some(RowNode::Leaf { row_id, row_hash, .. }) => {
                // Verify the row_id matches
                if *row_id != target_row_id {
                    return None;
                }

                // Pack final nibble into path
                if depth < key.len() {
                    self.pack_nibble(path, key[depth]);
                }

                Some(*row_hash)
            }
            Some(RowNode::Branch { children, .. }) => {
                if depth >= key.len() {
                    return None;
                }

                let nibble = key[depth];
                self.pack_nibble(path, nibble);

                // Build bitmap and collect siblings
                let mut bitmap = 0u16;
                let mut siblings = Vec::new();

                for (i, child) in children.iter().enumerate() {
                    if child.is_some() {
                        bitmap |= 1 << i;
                        if i != nibble as usize {
                            siblings.push(child.as_ref().unwrap().hash());
                        }
                    }
                }

                levels.push(ProofLevel { bitmap, siblings });

                // Continue down the path
                self.do_generate_hexary_proof(
                    children[nibble as usize].as_ref().map(|c| c.as_ref()),
                    key,
                    depth + 1,
                    levels,
                    path,
                    target_row_id,
                )
            }
            Some(RowNode::Extension { prefix, child, .. }) => {
                // Flatten: add prefix nibbles directly to path
                for &nibble in prefix {
                    self.pack_nibble(path, nibble);
                }

                // Continue to child without adding a proof level
                self.do_generate_hexary_proof(
                    Some(child.as_ref()),
                    key,
                    depth + prefix.len(),
                    levels,
                    path,
                    target_row_id,
                )
            }
        }
    }
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_row_trie_get_hexary_proof_basic --lib`
Expected: PASS

**Step 5: Add test for multiple rows**

```rust
#[test]
fn test_row_trie_get_hexary_proof_multiple() {
    use crate::determ::{DetermRow, DetermValue};
    use crate::trie::row_trie::RowTrie;

    let mut trie = RowTrie::new();

    let row1 = DetermRow::from_values(vec![DetermValue::integer(1)]);
    let row2 = DetermRow::from_values(vec![DetermValue::integer(2)]);

    trie.insert(1, row1);
    trie.insert(2, row2);

    // Both rows should have valid proofs
    let proof1 = trie.get_hexary_proof(1);
    assert!(proof1.is_some());
    assert!(proof1.unwrap().verify());

    let proof2 = trie.get_hexary_proof(2);
    assert!(proof2.is_some());
    assert!(proof2.unwrap().verify());

    // Non-existent row
    let proof3 = trie.get_hexary_proof(999);
    assert!(proof3.is_none());
}
```

**Step 6: Run test to verify it passes**

Run: `cargo test test_row_trie_get_hexary_proof_multiple --lib`
Expected: PASS

**Step 7: Commit**

```bash
git add src/trie/row_trie.rs
git commit -m "feat(trie): implement get_hexary_proof in RowTrie

- Generate hexary proofs from RowTrie
- Collect bitmap and sibling hashes at each branch
- Flatten extension nodes (add prefix to path)
- Pack nibbles 2-per-byte for compact paths
- Verify proof generation works for single and multiple rows"
```

---

## Task 7: Implement Solana-Style Serialization

**Files:**
- Modify: `src/trie/proof.rs`

**Step 1: Write failing test for serialization**

```rust
#[test]
fn test_hexary_proof_serialization_roundtrip() {
    use crate::trie::proof::{HexaryProof, ProofLevel, SolanaSerialize};

    let original = HexaryProof {
        value_hash: [1u8; 32],
        levels: vec![
            ProofLevel {
                bitmap: 0x1234,
                siblings: vec![[2u8; 32], [3u8; 32]],
            },
        ],
        root: [4u8; 32],
        path: vec![0x35, 0xC3],
    };

    let serialized = original.serialize();
    let deserialized = HexaryProof::deserialize(&serialized).unwrap();

    assert_eq!(original, deserialized);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_hexary_proof_serialization_roundtrip --lib`
Expected: FAIL with "no method named serialize"

**Step 3: Add SolanaSerialize trait and implement for HexaryProof**

Add to src/trie/proof.rs:

```rust
/// Error type for serialization/deserialization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializationError {
    InsufficientData { expected: usize, found: usize },
    InvalidData(String),
}

/// Trait for Solana-style serialization
pub trait SolanaSerialize: Sized {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> std::result::Result<Self, SerializationError>;
}
```

Then implement for HexaryProof (add helper to convert SerializationError to crate Error later):

```rust
impl SolanaSerialize for HexaryProof {
    fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();

        // Value hash (32 bytes)
        out.extend_from_slice(&self.value_hash);

        // Root hash (32 bytes)
        out.extend_from_slice(&self.root);

        // Path length (u8) + path data
        out.push(self.path.len() as u8);
        out.extend_from_slice(&self.path);

        // Number of levels (u8)
        out.push(self.levels.len() as u8);

        // Each level: bitmap (u16 LE) + sibling count (u8) + sibling hashes
        for level in &self.levels {
            out.extend_from_slice(&level.bitmap.to_le_bytes());
            out.push(level.siblings.len() as u8);
            for sibling in &level.siblings {
                out.extend_from_slice(sibling);
            }
        }

        out
    }

    fn deserialize(data: &[u8]) -> std::result::Result<Self, SerializationError> {
        use SerializationError::*;

        let mut cursor = 0;

        // Value hash
        if data.len() < cursor + 32 {
            return Err(InsufficientData { expected: cursor + 32, found: data.len() });
        }
        let value_hash = data[cursor..cursor + 32].try_into().unwrap();
        cursor += 32;

        // Root hash
        if data.len() < cursor + 32 {
            return Err(InsufficientData { expected: cursor + 32, found: data.len() });
        }
        let root = data[cursor..cursor + 32].try_into().unwrap();
        cursor += 32;

        // Path length
        if data.len() < cursor + 1 {
            return Err(InsufficientData { expected: cursor + 1, found: data.len() });
        }
        let path_len = data[cursor] as usize;
        cursor += 1;

        // Path data
        if data.len() < cursor + path_len {
            return Err(InsufficientData { expected: cursor + path_len, found: data.len() });
        }
        let path = data[cursor..cursor + path_len].to_vec();
        cursor += path_len;

        // Levels length
        if data.len() < cursor + 1 {
            return Err(InsufficientData { expected: cursor + 1, found: data.len() });
        }
        let levels_len = data[cursor] as usize;
        cursor += 1;

        let mut levels = Vec::new();
        for _ in 0..levels_len {
            // Bitmap (u16 LE)
            if data.len() < cursor + 2 {
                return Err(InsufficientData { expected: cursor + 2, found: data.len() });
            }
            let bitmap = u16::from_le_bytes(data[cursor..cursor + 2].try_into().unwrap());
            cursor += 2;

            // Sibling count
            if data.len() < cursor + 1 {
                return Err(InsufficientData { expected: cursor + 1, found: data.len() });
            }
            let sibling_count = data[cursor] as usize;
            cursor += 1;

            // Sibling hashes
            let mut siblings = Vec::new();
            for _ in 0..sibling_count {
                if data.len() < cursor + 32 {
                    return Err(InsufficientData { expected: cursor + 32, found: data.len() });
                }
                siblings.push(data[cursor..cursor + 32].try_into().unwrap());
                cursor += 32;
            }

            levels.push(ProofLevel { bitmap, siblings });
        }

        Ok(HexaryProof {
            value_hash,
            levels,
            root,
            path,
        })
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_hexary_proof_serialization_roundtrip --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/trie/proof.rs
git commit -m "feat(trie): implement Solana-style serialization for HexaryProof

- Add SolanaSerialize trait
- Binary format: value_hash, root, path, levels
- Each level: bitmap, sibling_count, sibling_hashes
- Roundtrip serialization test"
```

---

## Task 8: Implement Batch Verification

**Files:**
- Modify: `src/trie/proof.rs`, `Cargo.toml`

**Step 1: Add rayon dependency**

Run: `cargo add rayon --dev` (for development only initially, can enable later)

Or manually add to Cargo.toml:
```toml
[dependencies]
rayon = { version = "1.11", optional = true }

[features]
default = ["parallel"]
parallel = ["rayon"]
```

**Step 2: Write failing test for batch verification**

```rust
#[test]
fn test_hexary_proof_batch_verify() {
    use crate::trie::proof::{HexaryProof, ProofLevel};

    let proofs = vec![
        HexaryProof {
            value_hash: [1u8; 32],
            levels: vec![],
            root: [1u8; 32],
            path: vec![],
        },
        HexaryProof {
            value_hash: [2u8; 32],
            levels: vec![],
            root: [2u8; 32],
            path: vec![],
        },
    ];

    assert!(HexaryProof::verify_batch(&proofs));

    // With invalid proof
    let invalid_proofs = vec![
        HexaryProof {
            value_hash: [1u8; 32],
            levels: vec![],
            root: [99u8; 32], // Wrong
            path: vec![],
        },
    ];

    assert!(!HexaryProof::verify_batch(&invalid_proofs));
}
```

**Step 3: Run test to verify it fails**

Run: `cargo test test_hexary_proof_batch_verify --lib --features parallel`
Expected: FAIL with "no method named verify_batch"

**Step 4: Implement batch verification**

Add to impl HexaryProof:

```rust
    /// Verify multiple proofs in parallel
    ///
    /// Uses rayon for parallel verification across CPU cores.
    /// Returns true only if ALL proofs are valid.
    ///
    /// # Arguments
///
    /// * `proofs` - Slice of proofs to verify
    ///
    /// # Returns
    ///
    /// `true` if all proofs are valid, `false` if any is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use stoolap::trie::proof::HexaryProof;
    ///
    /// let proofs = vec![proof1, proof2, proof3];
    /// if HexaryProof::verify_batch(&proofs) {
    ///     println!("All proofs valid!");
    /// }
    /// ```
    #[cfg(feature = "parallel")]
    pub fn verify_batch(proofs: &[Self]) -> bool {
        use rayon::prelude::*;
        proofs.par_iter().all(|p| p.verify())
    }

    /// Verify multiple proofs sequentially
    ///
    /// Single-threaded version for environments without rayon.
    ///
    /// # Arguments
    ///
    /// * `proofs` - Slice of proofs to verify
    ///
    /// # Returns
    ///
    /// `true` if all proofs are valid, `false` otherwise
    pub fn verify_batch_sequential(proofs: &[Self]) -> bool {
        proofs.iter().all(|p| p.verify())
    }
```

**Step 5: Run test to verify it passes**

Run: `cargo test test_hexary_proof_batch_verify --lib --features parallel`
Expected: PASS

**Step 6: Commit**

```bash
git add src/trie/proof.rs Cargo.toml
git commit -m "feat(trie): add parallel batch verification for HexaryProof

- Add verify_batch with rayon for parallel verification
- Add verify_batch_sequential for single-threaded environments
- Add 'parallel' feature flag
- Tests for batch verification"
```

---

## Task 9: Remove Deprecated MerkleProof

**Files:**
- Modify: `src/trie/proof.rs`, `src/trie/tests/proof_tests.rs`

**Step 1: Update documentation and add deprecation notice**

Add to MerkleProof:
```rust
/// A Merkle proof that demonstrates inclusion of a value in a Merkle tree
///
/// # DEPRECATED
///
/// This type is designed for binary trees and is not compatible with
/// the 16-way hexary RowTrie. Use `HexaryProof` instead.
///
/// # Fields
///
/// * `value_hash` - The hash of the value being proven
/// * `siblings` - The sibling hashes needed to reconstruct the root
/// * `root` - The expected Merkle root
/// * `path` - The path (indices) from root to leaf in the tree
#[deprecated(since = "0.3.0", note = "Use HexaryProof for hexary tries instead")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProof {
    // ... existing fields ...
}
```

**Step 2: Run tests to ensure nothing breaks**

Run: `cargo test --lib`

**Step 3: Remove MerkleProof entirely**

Remove the entire MerkleProof struct and its impl block.

**Step 4: Update tests that use MerkleProof**

Remove or update tests in `src/trie/tests/proof_tests.rs` to use HexaryProof instead.

**Step 5: Run tests to verify**

Run: `cargo test --lib`

**Step 6: Commit**

```bash
git add src/trie/proof.rs src/trie/tests/proof_tests.rs
git commit -m "refactor(trie): remove deprecated MerkleProof

- Remove binary-tree MerkleProof type
- Replace with HexaryProof throughout
- Update all tests to use HexaryProof"
```

---

## Task 10: Update Integration Tests

**Files:**
- Modify: `tests/blockchain_integration_test.rs`

**Step 1: Run integration tests**

Run: `cargo test --test blockchain_integration_test`

**Step 2: Fix any failing tests**

Update tests to use HexaryProof if needed.

**Step 3: Commit**

```bash
git add tests/blockchain_integration_test.rs
git commit -m "test: update integration tests for HexaryProof"
```

---

## Task 11: Add Benchmarks

**Files:**
- Create: `benches/hexary_proof.rs`

**Step 1: Create benchmark file**

```rust
// Copyright 2025 Stoolap Contributors
// Licensed under the Apache License, Version 2.0

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use stoolap::determ::{DetermRow, DetermValue};
use stoolap::trie::row_trie::RowTrie;

fn bench_proof_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_generation");

    for size in [10, 100, 1000].iter() {
        let mut trie = RowTrie::new();
        for i in 0..*size {
            let row = DetermRow::from_values(vec![DetermValue::integer(i)]);
            trie.insert(i, row);
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let proof = trie.get_hexary_proof(black_box(size / 2));
                black_box(proof);
            });
        });
    }

    group.finish();
}

fn bench_proof_verification(c: &mut Criterion) {
    let mut trie = RowTrie::new();
    for i in 0..100 {
        let row = DetermRow::from_values(vec![DetermValue::integer(i)]);
        trie.insert(i, row);
    }

    let proof = trie.get_hexary_proof(50).unwrap();

    c.bench_function("verify_single_proof", |b| {
        b.iter(|| {
            black_box(&proof).verify();
        });
    });
}

fn bench_proof_serialization(c: &mut Criterion) {
    let mut trie = RowTrie::new();
    for i in 0..100 {
        let row = DetermRow::from_values(vec![DetermValue::integer(i)]);
        trie.insert(i, row);
    }

    let proof = trie.get_hexary_proof(50).unwrap();
    let serialized = proof.serialize();

    c.bench_function("serialize_proof", |b| {
        b.iter(|| {
            black_box(&proof).serialize();
        });
    });

    c.bench_function("deserialize_proof", |b| {
        b.iter(|| {
            HexaryProof::deserialize(black_box(&serialized)).unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_proof_generation,
    bench_proof_verification,
    bench_proof_serialization
);
criterion_main!(benches);
```

**Step 2: Add criterion to dev dependencies**

Add to Cargo.toml:
```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "hexary_proof"
harness = false
```

**Step 3: Run benchmarks**

Run: `cargo bench --bench hexary_proof`

**Step 4: Commit**

```bash
git add benches/hexary_proof.rs Cargo.toml
git commit -m "bench: add HexaryProof benchmarks

- Proof generation for various trie sizes
- Single proof verification
- Serialization/deserialization"
```

---

## Task 12: Final Documentation and Cleanup

**Files:**
- Modify: `src/trie/proof.rs`, `src/trie/mod.rs`

**Step 1: Update module documentation**

Update `src/trie/mod.rs` to export HexaryProof prominently:

```rust
//! Merkle trie implementations
//!
//! This module provides hexary Merkle tries for efficient data storage
//! and verification in the blockchain.
//!
//! # HexaryProof
//!
//! The `HexaryProof` type provides compact, verifiable proofs for
//! data inclusion in the 16-way hexary trie. Proofs use:
//! - Bitmap-based sibling encoding for compactness
//! - Nibble-packed paths for efficiency
//! - SHA-256 for cryptographic security
//!
//! # Example
//!
//! ```
//! use stoolap::trie::row_trie::RowTrie;
//!
//! let mut trie = RowTrie::new();
//! // ... insert data ...
//!
//! let proof = trie.get_hexary_proof(row_id)?;
//! if proof.verify() {
//!     println!("Proof valid!");
//! }
//! ```

pub mod proof;
pub mod row_trie;
pub mod schema_trie;

pub use proof::{HexaryProof, ProofLevel};
pub use row_trie::RowTrie;
pub use schema_trie::SchemaTrie;
```

**Step 2: Run full test suite**

Run: `cargo test --all-targets`

**Step 3: Run benchmarks**

Run: `cargo bench`

**Step 4: Final commit**

```bash
git add src/trie/mod.rs
git commit -m "docs: update trie module documentation for HexaryProof

- Export HexaryProof and ProofLevel prominently
- Add module-level documentation with examples
- Finalize hexary proof implementation"
```

---

## Summary

This implementation plan:
1. Adds `HexaryProof` and `ProofLevel` data structures
2. Implements nibble packing/unpacking for compact paths
3. Implements bitmap-based sibling reconstruction
4. Implements 16-way child hashing
5. Implements streaming verification
6. Implements proof generation in RowTrie
7. Implements Solana-style serialization
8. Implements parallel batch verification
9. Removes deprecated `MerkleProof`
10. Updates all tests and documentation

**Estimated completion time:** 12-15 tasks × 2-5 minutes each = ~1-2 hours of focused work

**Test coverage:** Unit tests for each component, integration tests, benchmarks

**Success criteria:**
- All tests pass
- Benchmarks show proof size <100 bytes for typical cases
- Verification <5 μs per proof
- Batch verification scales linearly with thread count
