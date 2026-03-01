# Stoolap Dependencies

## Core Dependencies

### STWO - Zero-Knowledge Proofs
**Version:** 2.1
**Optional:** Yes (requires `zk` feature)
**Purpose:** Circle STARK prover/verifier for Cairo programs

**Repository:** https://github.com/starkware-libs/stwo-cairo

**Features:**
- STARK proof generation
- STARK proof verification
- Cairo program execution
- Circle STARK cryptographic primitives

**Usage:**
```toml
[dependencies]
stoolap = { version = "0.3", features = ["zk"] }
```

**Integration Status:**
- ✅ Mission 0201-01: STWO Dependency Integration (Complete)
- 📋 Mission 0201-02: Cairo Program Types (Open)
- 📋 Mission 0201-03: Cairo Compiler Integration (Open)

## Related Dependencies

### blake3
**Version:** 1.5
**Optional:** Yes (requires `zk` feature)
**Purpose:** Fast hashing for Cairo program identifiers

### sha2
**Version:** 0.10.9
**Purpose:** SHA-256 for Merkle tree hashing (RFC-0101)

## Optional Features

### `zk` - Zero-Knowledge Proofs
Enables STWO integration for:
- Proof compression (RFC-0202)
- Confidential queries (RFC-0203)
- L2 rollup (RFC-0204)

```bash
cargo build --features zk
cargo test --features zk
```

## Build Status

| Feature | Status | Tests |
|---------|--------|-------|
| zk (STWO) | ✅ Implemented | 4/4 passing |
| sha2 | ✅ Implemented | All passing |
