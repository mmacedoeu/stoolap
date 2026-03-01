# Mission: Trie Extension Key Slicing Fix

## Status
Open

## RFC
RFC-0105

## Acceptance Criteria

- [ ] Modify `do_get_hash()` Extension case to use `key.starts_with(&prefix)` and `&key[prefix.len()]`
- [ ] Modify `do_get()` Extension case to use the same key slicing logic
- [ ] All existing tests continue to pass (24/27 tests)
- [ ] The 3 failing sequential row tests now pass:
  - [ ] `test_compressed_proof_sequential_rows_10`
  - [ ] `test_compressed_proof_sequential_rows_large`
  - [ ] `test_compressed_proof_mixed_ids`
- [ ] Add regression test for sequential row IDs 1-100
- [ ] Verify no performance regression in benchmarks

## Claimant
None

## Pull Request
#

## Notes

### Root Cause
Extension node lookup uses depth-based checking (`key[depth..depth + prefix.len()]`) while insertion uses prefix-based checking (`key.starts_with(&prefix)`). Only insertion slices the key.

### Files to Modify
1. `src/trie/row_trie.rs`:
   - `do_get_hash()` function, Extension case (~line 686)
   - `do_get()` function, Extension case (~line 730)

### Test File
- `src/trie/tests/row_trie_test.rs` or `src/trie/row_trie.rs` (test module)

### Verification Command
```bash
cargo test --package stoolap-chain --lib trie::tests
cargo test --package stoolap-chain --lib row_trie
```

### Expected Result
27/27 tests passing (currently 24/27)

### Context
This fix unblocks mission 0202-02 (Compressed Proof Generation) which is currently blocked on sequential row lookup failures.
