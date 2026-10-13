# bitmap256

A pure Rust library implementing a fixed-size 256-bit bitmap: a compact, stack-allocated bitset for tracking flags, permissions, membership, or presence.

Built for performance-critical code where allocation-free operations matter: no heap, no generics, no monomorphization bloat. Just four `u64` words and a clean, zero-cost API.

## Motivation

When you need a large bitset (e.g., capability masks, permission flags, active-session tracking) and `bitvec::BitVec` or `std::collections::HashSet` is too heavy, `bitmap256` gives you the full 256-bit range with zero allocation and predictable performance.

## Features

- **Fixed Size**: Exactly 256 bits (4 × `u64`), stored compactly on the stack.
- **Operator Overloading**: `<<`, `>>`, `^`, `!` for intuitive bitwise manipulation.
- **Iteration**: `IntoIterator` yields indices of set bits in ascending order.
- **Utility Methods**:
  - `count_ones()` — population count
  - `leading_zeros()` / `trailing_zeros()` — fast zero detection
  - `most_significant_bit()` / `least_significant_bit()` — index retrieval
- **Zero Allocation**: Entirely stack-based; no heap usage.
- **Full Tests**: 44+ unit tests covering all edge cases and boundary conditions.

## Performance

All operations are stack-allocated and branchless where possible. Benchmarked on a typical modern CPU:

| Operation | Time |
|-----------|------|
| `Bitmap256::new()` | ~1 ns |
| `Bitmap256::from_value()` | ~0.7 ns |
| `Bitmap256::from_bit(n)` | ~6 ns |
| `set_bit(n)` / `clear_bit(n)` / `flip_bit(n)` | ~5 ns |
| `count_ones()` | ~2 ns |
| `leading_zeros()` / `trailing_zeros()` | <1 ns |
| `most_significant_bit()` / `least_significant_bit()` | ~1 ns |
| `has_bit(n)` | ~1 ns |
| `ones()` (sparse, 3 bits) | ~7 ns |
| `ones()` (dense, 64 bits) | ~170 ns |
| `ones()` (full, 256 bits) | ~775 ns |
| `a << n` / `a >> n` | ~6 ns |
| `a ^ b` | ~1.7 ns |
| `!a` | ~1 ns |

*Results are per-operation averages from Criterion benchmarks in `benches/benchmarks.rs`.*

## Bit Layout & Indexing

Bits are indexed from **0** (LSB of the last word) to **255** (MSB of the first word):

| Index Range | Stored In | Word Index |
|-------------|-----------|------------|
| 0–63        | `u64[3]`  | Last       |
| 64–127      | `u64[2]`  | Third      |
| 128–191     | `u64[1]`  | Second     |
| 192–255     | `u64[0]`  | First      |

> **Note**: `from_value()` only populates words 2 and 3 (indices 0–127). Words 0 and 1 remain zero regardless of input.

## Usage

```rust
use bitmap256::Bitmap256;

let mut bits = Bitmap256::new();
bits.set_bit(10);
bits.set_bit(200);

assert_eq!(bits.count_ones(), 2);

for index in &bits {
    println!("Set bit at: {}", index);
}
// Set bit at: 10
// Set bit at: 200
```

### Construction

```rust
// From a primitive — lower bits map to lower indices
let bitmap = Bitmap256::from_value(0xFFu128);
assert!(bitmap.has_bit(0));   // bit 0 set
assert!(bitmap.has_bit(7));   // bit 7 set

// Single bit
let flag = Bitmap256::from_bit(42);

// Shift a single bit into position
let shifted = Bitmap256::from_bit(0) << 128;
```

### Bitwise Operations

```rust
let a = Bitmap256::from_bit(10);
let b = Bitmap256::from_bit(200);

let combined = a ^ b;          // XOR: both bits set
let inverted = !a;             // All bits flipped

let shifted = a << 5;          // Left shift by 5
```

## Why Not `bitvec`?

| Criterion | `bitmap256` | `bitvec::BitVec` |
|-----------|-------------|-------------------|
| Heap alloc  | No | Yes (by default) |
| Generics    | No | Yes |
| Max bits    | 256 | Arbitrary |
| Overflow    | Panics | Panics |
| Compile time | Fast | Slowest |

## When to Use

- Fixed-size flag sets (e.g., file permission bits, capability masks).
- Presence tracking (e.g., active sessions, online users).
- Any scenario where `std::collections::HashSet` or `bitvec::BitVec` is overkill and introduces unwanted allocation or indirection.

## License

MIT / Apache-2.0

---

### Transparency Note

The implementation of this library is entirely hand-written. The documentation and README were generated with assistance from **Qwen 3.6 35B** and subsequently verified and refined by hand to ensure accuracy and completeness.
