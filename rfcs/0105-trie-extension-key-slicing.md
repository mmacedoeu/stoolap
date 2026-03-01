# RFC-0105: Trie Extension Key Slicing Fix

## Status
Draft

## Summary

Fixes the key slicing inconsistency in hexary trie extension node traversal. The lookup operation must slice keys when traversing extensions, matching the behavior of insertion operations.

## Motivation

The current trie implementation has an asymmetry between insertion and lookup:

| Operation | Prefix Check | Key Slicing |
|-----------|--------------|-------------|
| Insertion | `key.starts_with(&prefix)` | `&key[prefix.len()]` |
| Lookup | `key[depth..depth + prefix.len()]` | No slicing |

This causes lookup failures for sequential row IDs (1, 2, 3, ...) because:
1. Sequential IDs encode to keys starting with the same nibble `[0]`
2. After inserting rows 1-2, extensions are created with prefix `[0]`
3. Insertion slices past the prefix, continuing with the remaining key
4. Lookup does NOT slice, so it tries to navigate using the original key at an offset
5. This causes lookup to check the wrong child branch

## Specification

### The Fix

Modify the `do_get_hash` function's Extension case to match insertion behavior:

**Before:**
```rust
Some(RowNode::Extension { prefix, child, .. }) => {
    // Check if the key starting at depth has the extension's prefix
    if depth + prefix.len() <= key.len() {
        let key_prefix = &key[depth..depth + prefix.len()];
        if key_prefix == &prefix[..] {
            return self.do_get_hash(
                Some(child.as_ref()),
                key,  // Don't slice the key
                depth + prefix.len(),
                target_row_id,
            );
        }
    }
    None
}
```

**After:**
```rust
Some(RowNode::Extension { prefix, child, .. }) => {
    // Check if current key starts with extension's prefix (matching insertion behavior)
    if key.starts_with(&prefix) {
        return self.do_get_hash(
            Some(child.as_ref()),
            &key[prefix.len()..],  // Slice the key (matching insertion)
            depth + prefix.len(),
            target_row_id,
        );
    }
    None
}
```

### Affected Functions

The fix must be applied to two functions that traverse extensions:

1. **`do_get_hash`** - Used by `get_hash()` for proof generation
2. **`do_get`** - Used by `get()` for row retrieval

Both functions must use the same key slicing logic.

### Invariant

After the fix, the following invariant must hold:

> For any row ID X: if `insert(X, value)` succeeds, then `get(X)` returns `value`

## Rationale

### Why This Approach?

1. **Minimal Change** - Only modifies lookup, not insertion or data structures
2. **Matches Working Code** - Insertion already works correctly; lookup should match
3. **Consistent Semantics** - Both operations use the same prefix checking and slicing

### Alternatives Considered

| Alternative | Pros | Cons | Chosen? |
|------------|------|------|---------|
| A: Slice during lookup | Minimal, matches insertion | None identified | ✅ |
| B: Use depth-based checking in insertion | More "explicit" | Requires changing insertion (working code) | ❌ |
| C: Avoid sequential IDs | No code change | Not user-controllable, limits functionality | ❌ |

## Implementation

### Files Modified

- `src/trie/row_trie.rs`:
  - `RowTrie::do_get_hash()` - Extension case (around line 686)
  - `RowTrie::do_get()` - Extension case (around line 730)

### Testing Requirements

- All existing tests must continue to pass
- The 3 failing sequential row tests in mission 0202-02 must now pass
- Add regression test for sequential row IDs 1-100
- Verify extension node traversal in various trie configurations

### Dependencies

- Requires: RFC-0101 (Hexary Merkle Proofs)
- Enables: RFC-0202 (Compressed Proofs) - unblocks mission 0202-02

## Performance Considerations

- **Time complexity**: No change - still O(depth) for lookup
- **Space complexity**: No change - same number of recursive calls
- **Performance**: Identical to current implementation (slicing is cheap)

## Security Considerations

- **Correctness**: This is a security fix—incorrect lookups could return wrong data
- **Determinism**: Fix preserves deterministic behavior (same inputs → same outputs)
- **Attack surface**: Reduces attack surface by removing incorrect behavior

## Backward Compatibility

- **Breaking changes**: None—this fixes broken behavior
- **Migration path**: None needed—bug fix is transparent to users
- **Deprecation timeline**: N/A

## Related Use Cases

- [Trie Sequential Row Lookups](../../docs/use-cases/trie-sequential-lookups.md)

## Related RFCs

- [RFC-0101: Hexary Merkle Proofs](./0101-hexary-merkle-proofs.md)
- [RFC-0202: Compressed Proofs](./0202-compressed-proofs.md)

## Open Questions

None—the fix is straightforward and well-defined.

## References

- [Economic Encoding Specifications](https://eth.wiki/en/fundamentals/patricia-tree#economic-encoding-for-hex-patricia-tries)
- Current code: `src/trie/row_trie.rs` lines 686-700 (do_get_hash), lines 730-750 (do_get)
