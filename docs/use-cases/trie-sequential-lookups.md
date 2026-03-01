# Use Case: Trie Sequential Row Lookups

## Problem

The hexary trie implementation fails to correctly retrieve sequential row IDs beyond the first two entries. When inserting rows with sequential IDs (1, 2, 3, ...), the trie can successfully retrieve rows 1 and 2, but lookup fails for row 3 and beyond.

This is a critical correctness bug that violates the fundamental contract of a key-value store: **if you insert a value, you must be able to retrieve it**.

### Root Cause

The bug stems from inconsistent key slicing between insertion and lookup operations in extension node traversal:

- **Insertion**: Uses `key.starts_with(&prefix)` check and slices key with `&key[prefix.len()]`
- **Lookup**: Uses `key[depth..depth + prefix.len()]` check but does NOT slice the key

This asymmetry causes lookup to navigate incorrect paths when sequential row IDs share prefix nibbles.

## Motivation

Fixing this bug is essential for:

### Correctness
- The database must return correct results for all valid row IDs
- Users cannot work around this by avoiding sequential IDs—many natural use cases generate sequential data
- A database that cannot retrieve stored data is fundamentally broken

### Compressed Proof Generation (RFC-0202)
- Mission 0202-02 (Compressed Proof Generation) requires batch row lookup by ID
- Currently, 24/27 tests pass—3 tests fail specifically due to sequential row lookup failures
- The mission cannot be completed without this fix

### Trust in the Protocol
- A correctness bug in core data structures undermines trust in the entire system
- Blockchain systems require deterministic, verifiable behavior
- Data integrity is non-negotiable

## Impact

If this bug is fixed:

1. **Mission 0202-02 can complete** - All 27 tests will pass
2. **Batch operations work correctly** - Sequential row IDs are common in real-world scenarios
3. **Data integrity is maintained** - All inserted rows become retrievable
4. **Proof generation is reliable** - Compressed proofs can include any sequence of rows

## Related RFCs

- [RFC-0101: Hexary Merkle Proofs](../../rfcs/0101-hexary-merkle-proofs.md) - Trie structure definition
- [RFC-0202: Compressed Proofs](../../rfcs/0202-compressed-proofs.md) - Depends on correct trie lookups

## Non-Goals

- This does NOT change the trie data structure design
- This does NOT add new features
- This is purely a correctness fix to match insertion behavior

## Success Criteria

- All 27 tests in mission 0202-02 pass
- Sequential row IDs (1, 2, 3, ..., N) are all retrievable after insertion
- No existing tests are broken by the fix
- Lookup behavior matches insertion behavior exactly
