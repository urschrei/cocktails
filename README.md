# Calculating an Optimal Cocktail Ingredient List

## [Tom](https://gist.github.com/tmcw/c6bdcfe505057ed6a0f356cfd02d4d52) Explains the Problem

- You have 100 different ingredients
- You have 20 cocktails, each of which use 2-6 ingredients
- Which 5 ingredients maximize the cocktail-making possibilities? What about 10 ingredients?

In this case, we have **117 ingredients**, and **104 available cocktails**.
Can we work out an "ideal" list of cocktail ingredients of an **arbitrary length** that will maximise the number of unique cocktails that we can make?

This is – in an abstract sense – a classic combinatorial optimisation problem. One way of solving combinatorics problems is ["branch and bound"](https://en.wikipedia.org/wiki/Branch_and_bound) (BnB). This is a Rust implementation of the solution provided by Forest Gregg (reproduced here, lightly edited). A detailed explanation of how the algorithm works is detailed [here](branch_and_bound_explanation.md).

The Rust version uses a BitSet implementation using SmallVec-backed storage for fast bitwise operations.

## Running the code

Build the optimized version:
```bash
cargo build --release
```

Run with default settings (12 ingredients):
```bash
./target/release/branchbound
```

### Command-Line Options

```bash
Cocktail Ingredients Optimiser - Find optimal ingredient combinations

Usage: branchbound [OPTIONS]

Options:
  -i, --ingredients <INGREDIENTS>  Number of ingredients to select (2-109) [default: 12]
  -m, --max-calls <MAX_CALLS>      Maximum search iterations [default: 8000000]
  -f, --format <FORMAT>            Output format: table, json, or simple [default: table]
  -h, --help                       Print help
  -V, --version                    Print version
```

### Usage Examples

Find optimal ingredients for different counts:
```bash
./target/release/branchbound --ingredients 8   # Fast: ~15 ms
./target/release/branchbound --ingredients 12  # Medium: ~307 ms
./target/release/branchbound --ingredients 16  # Slower: ~6 s
```

Different output formats:
```bash
./target/release/branchbound --format table    # Pretty table (default)
./target/release/branchbound --format simple   # Minimal output
./target/release/branchbound --format json     # JSON for programmatic use
```


## Performance

The optimized Rust implementation scaling characteristics:

| Ingredients | Time | Iterations | Cocktails Found |
|-------------|------|------------|----------------|
| 8           | ~15ms | ~3,500     | 6 cocktails    |
| 12          | ~307ms | ~109,000   | 10 cocktails   |
| 16          | ~6s  | ~2,000,000 | 14 cocktails   |

Compared to the original Python implementation (12 ingredients):
- **Rust (optimized)**: 307ms 
- **Python**: 2.37 seconds
- **Speedup**: **7.7x faster**

The optimization involved:
1. Replacing `BTreeSet` operations with a custom `BitSet` using `SmallVec<[u64; 3]>` storage
2. Using fast bitwise operations for set union, intersection, and difference
3. Optimizing data structures with SmallVec to reduce heap allocations in hot paths
4. Building lookup tables for ingredient/cocktail indexing
5. Implementing in-place BitSet operations to reduce cloning

## Implementation Notes

### Scalability
The current implementation supports **unlimited ingredients** by using dynamic chunk allocation with SmallVec. The first 192 bits (3 × 64) are stored inline without heap allocation.

### For Larger Domains
For problems with more than 128 elements, consider using:
- **[bit-set crate](https://crates.io/crates/bit-set)**: Dynamic bit vectors with good performance
- **[Indexical crate](https://crates.io/crates/indexical)**: Provides object-to-index mapping with pluggable bit-set backends, including SIMD-optimized implementations

Both approaches sacrifice some performance for flexibility but remain much faster than standard `HashSet` or `BTreeSet` operations for set-intensive algorithms.

Both versions take around 100k iterations to converge on a 12-ingredient solution. While we previously used a random remaining candidate cocktail to test the quality of our current search – which resulted in a lot of "misses" – [we now use a new heuristic](https://github.com/fgregg/cocktails): the cocktail among the remaining candidates which is the "least unique" in its ingredients, calculated using a minimum amortized cost function. This has almost halved the number of search rounds, and produces an optimal solution for this heuristic:

1. Amaretto
2. Champagne
3. Cognac
4. Crème de cassis
5. Galliano
6. Gin
7. Grenadine
8. Lemon juice
9. Lime juice
10. Simple syrup
11. Triple sec
12. White rum

Allowing you to mix:

1. Bacardi cocktail
2. Between the sheets
3. Daiquiri
4. French 75
5. French Connection
6. Gimlet
7. Kir royal
8. Sidecar
9. White lady
10. Yellow bird
