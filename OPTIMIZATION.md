# Performance Optimization Guide

This document provides guidance on high-performance Rust patterns for the MTG Forge project, with a focus on zero-copy patterns and minimizing allocations.

## Current Performance Metrics

Our key performance metrics (as of 2025-10-26_#333(dc90c78b)):

**Fresh Mode (seed 42):**
- **Games/sec**: ~3,842
- **Actions/sec**: ~464,585
- **Turns/sec**: ~28,066
- **Actions/turn**: 16.56
- **Avg bytes/game**: ~233,426
- **Avg bytes/turn**: ~12,968
- **Avg duration/game**: 260.36µs

**Snapshot Mode (seed 42):**
- **Games/sec**: ~9,177
- **Actions/sec**: ~2,734,713
- **Avg bytes/game**: ~122,884
- **Avg duration/game**: 108.97µs

**Rewind Mode (seed 42):**
- **Rewinds/sec**: ~332,103
- **Actions/sec (rewind)**: ~107,686,651
- **Avg bytes allocated**: 0 (zero-copy)

**Note:** These metrics reflect a much richer game implementation compared to earlier versions.
The lower games/sec in fresh mode is offset by dramatically increased actions/turn (0.82 → 16.56),
showing that each game now involves substantially more gameplay. Actions/sec remains a good
normalized metric for raw engine performance.

## Zero-Copy Patterns and Best Practices

### 1. Avoid Unnecessary `clone()`

**Problem**: Cloning creates deep copies of data, which is expensive for large structures.

**Solution**: Use references and manage lifetimes appropriately.

```rust
// ❌ BAD: Unnecessary clone
fn process_cards(cards: &Vec<Card>) -> Vec<Card> {
    cards.clone()
}

// ✅ GOOD: Return reference or iterator
fn process_cards(cards: &Vec<Card>) -> &[Card] {
    cards.as_slice()
}

// ✅ EVEN BETTER: Return iterator for lazy evaluation
fn process_cards(cards: &Vec<Card>) -> impl Iterator<Item = &Card> {
    cards.iter().filter(|c| c.is_creature())
}
```

**When to use `.iter().cloned()` vs `.clone().iter()`**:
- `v.iter().cloned()` creates a borrowed iterator that clones items on-the-fly (no Vec allocation)
- `v.clone().iter()` clones the entire Vec first (expensive heap allocation)
- Always prefer `v.iter().cloned()` when you need owned values from iteration

### 2. Avoid Unnecessary `collect()`

**Problem**: Calling `collect()` allocates a new collection when the data might only be iterated once.

**Solution**: Return iterator types (`impl Iterator<Item=T>`) instead of `Vec<T>`.

```rust
// ❌ BAD: Unnecessary collect
fn get_creatures(cards: &[Card]) -> Vec<&Card> {
    cards.iter().filter(|c| c.is_creature()).collect()
}

// ✅ GOOD: Return iterator
fn get_creatures(cards: &[Card]) -> impl Iterator<Item = &Card> + '_ {
    cards.iter().filter(|c| c.is_creature())
}
```

### 3. Chain Iterator Operations

**Problem**: Multiple `collect()` calls between operations create temporary collections.

**Solution**: Chain iterator methods together for a single traversal.

```rust
// ❌ BAD: Multiple collects
let creatures: Vec<_> = cards.iter().filter(|c| c.is_creature()).collect();
let tapped: Vec<_> = creatures.iter().filter(|c| c.is_tapped()).collect();

// ✅ GOOD: Chained operations
let tapped_creatures = cards.iter()
    .filter(|c| c.is_creature())
    .filter(|c| c.is_tapped());
```

### 4. Use Slices Instead of Owned Types

**Problem**: Taking owned `String` or `Vec<T>` when you only need to read.

**Solution**: Use `&str` instead of `&String`, and `&[T]` instead of `&Vec<T>`.

```rust
// ❌ BAD: Unnecessary specificity
fn print_name(name: &String) { }
fn process_cards(cards: &Vec<Card>) { }

// ✅ GOOD: Use slices
fn print_name(name: &str) { }
fn process_cards(cards: &[Card]) { }
```

### 5. Implement `size_hint()` for Custom Iterators

**Problem**: Collections can't pre-allocate if they don't know the iterator size.

**Solution**: Implement `Iterator::size_hint()` or `ExactSizeIterator::len()` when possible.

```rust
impl Iterator for MyIterator {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> { /* ... */ }

    // Helps collect() and extend() pre-allocate
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining_count();
        (remaining, Some(remaining))
    }
}
```

### 6. Arena Allocation for Short-Lived Objects

**Problem**: Frequent small allocations and deallocations fragment memory and slow down the allocator.

**Solution**: Use arena allocators (like `bumpalo` or `typed-arena`) for per-frame or per-turn allocations.

```rust
// Consider for future optimization:
// - Per-turn arena for temporary combat calculations
// - Per-phase arena for stack resolution
// - Arena reset at phase/turn boundaries
```

**Benefits**:
- Allocation is just pointer increment (extremely fast)
- Deallocation is bulk operation (drop entire arena)
- Better cache locality (adjacent allocations)
- Particularly good for game engines with frame-based allocation patterns

### 7. Object Pools for Reusable Objects

**Problem**: Creating and destroying the same types of objects repeatedly (e.g., spell effects, combat damage calculations).

**Solution**: Pre-allocate a pool and reuse objects.

```rust
// Future consideration for:
// - Token pools
// - Effect pools
// - Combat calculation buffers
```

### 8. Use `SmallVec` and `SmallMap` for Expected-Small Collections

**Problem**: Many game entities have 0-2 counters/abilities but we allocate on the heap for any collection.

**Solution**: Use `smallvec::SmallVec` and similar crates to avoid heap allocation for small counts.

```rust
use smallvec::SmallVec;

// Stores up to 4 items inline, only heap-allocates if more
type CounterList = SmallVec<[Counter; 4]>;
```

**Already in use**: The project already uses `SmallVec` for counters (see PROJECT_VISION.md).

### 9. Prefer Unboxed Enums Over `Vec<Box<dyn Trait>>`

**Problem**: Java-style polymorphism with vectors of boxed trait objects creates pointer chasing and heap fragmentation.

**Solution**: Use enums with data variants when the set of types is closed.

```rust
// ❌ Less efficient: Boxed trait objects
Vec<Box<dyn Effect>>

// ✅ More efficient: Unboxed enum
enum Effect {
    DealDamage { target: EntityId, amount: u32 },
    DrawCards { player: PlayerId, count: u32 },
    // ... more variants
}
```

**Rust advantage**: Vectors of enums are stored contiguously without pointer indirection, unlike Java's object arrays.

### 10. Cow (Clone-on-Write) for Conditional Ownership

**Problem**: Sometimes you need owned data, sometimes borrowed, leading to unnecessary clones.

**Solution**: Use `std::borrow::Cow` to defer cloning until necessary.

```rust
use std::borrow::Cow;

fn process_name(name: Cow<str>) -> Cow<str> {
    if name.contains("transform") {
        Cow::Owned(name.to_uppercase()) // Only clone if needed
    } else {
        name // Return borrowed if no modification
    }
}
```

## Profiling and Measurement

### Running Benchmarks

```bash
# Run all benchmarks (slow)
make full-benchmark

# Run specific benchmarks (fast)
make bench-snapshot      # Snapshot benchmark only
make bench-logging       # Stdout logging benchmark only

# Or use cargo bench directly:
cargo bench --bench game_benchmark -- fresh
cargo bench --bench game_benchmark -- snapshot
cargo bench --bench game_benchmark -- fresh_stdout_logging
```

Key metrics to track:
- `Games/sec` and `Turns/sec` (absolute performance)
- `Actions/sec` (normalized metric that should stay stable)
- `Bytes/turn` (allocation pressure)

### Heap Profiling

```bash
# Generate heap profile
make heapprofile

# Process and view results
./scripts/analyze_heapprofile.sh
```

This will show the top allocation sites in your code with file:line references.

### CPU Profiling

```bash
# Generate flamegraph
make profile

# View the output
firefox flamegraph.svg
```

## Common Anti-Patterns to Avoid

### 1. Returning Fresh Collections

```rust
// ❌ BAD: Allocates new Vec every call
pub fn get_creatures(&self) -> Vec<CardId> {
    self.battlefield.iter()
        .filter(|c| c.is_creature())
        .map(|c| c.id)
        .collect()
}

// ✅ GOOD: Returns iterator over existing data
pub fn get_creatures(&self) -> impl Iterator<Item = CardId> + '_ {
    self.battlefield.iter()
        .filter(|c| c.is_creature())
        .map(|c| c.id)
}
```

### 2. Cloning to Satisfy the Borrow Checker

```rust
// ❌ BAD: Clone to avoid borrow checker
let cards = self.hand.clone();
self.do_something_that_mutates();
for card in cards { /* ... */ }

// ✅ GOOD: Collect IDs first (smaller), or restructure
let card_ids: Vec<_> = self.hand.iter().map(|c| c.id).collect();
self.do_something_that_mutates();
for id in card_ids {
    let card = self.get_card(id);
    /* ... */
}
```

### 3. Unnecessary String Allocations

```rust
// ❌ BAD: Creates temporary String
fn log_card(&self, card: &Card) {
    println!("Card: {}", card.name.clone());
}

// ✅ GOOD: Borrow string directly
fn log_card(&self, card: &Card) {
    println!("Card: {}", card.name);
}
```

### 4. Collecting Then Chaining

```rust
// ❌ BAD: Collect then iterate again
let creatures: Vec<_> = cards.iter().filter(|c| c.is_creature()).collect();
let untapped: Vec<_> = creatures.iter().filter(|c| !c.is_tapped()).collect();

// ✅ GOOD: Chain without intermediate collection
let untapped_creatures = cards.iter()
    .filter(|c| c.is_creature())
    .filter(|c| !c.is_tapped());
```

## Status and Backlog

### Known Inefficiencies

This section tracks identified allocation hotspots and optimization opportunities discovered through heap profiling (100 games, seed 42).

#### High Priority (From Profiling Results)

Based on `make heapprofile` analysis showing 228,016 total allocations across 100 games:

1. **String formatting in logging** - 77,378 calls, 304.54KB (src/game/game_loop.rs:819)
   - `Combat.clear()` triggers logging with `format!()` macros
   - Every end-of-combat step allocates strings for event logging
   - **Solution**: Use string interning, static strings, or conditional logging
   - **Impact**: ~34% of all allocations in our code

2. **Draw card logging** - 45,274 calls, 1.39KB (src/game/game_loop.rs:517)
   - `format!("{} draws {} ({})", player_name, card.name, card_id)`
   - Creates temporary string on every card draw
   - **Solution**: Lazy logging or pre-allocated string buffers
   - **Impact**: ~20% of all allocations in our code

3. **Discard logging** - 43,437 calls, 18.36KB (src/game/game_loop.rs:863)
   - `format!()` for discard notifications in cleanup step
   - **Solution**: Same as above - string interning or conditional logging
   - **Impact**: ~19% of all allocations in our code

4. **PlayerName Display trait** - 41,806 calls, 3.63KB (src/core/types.rs:871)
   - `write!(f, "{}", self.0)` in Display implementation
   - Called during every logging operation
   - **Solution**: Consider avoiding wrapper type or caching formatted names
   - **Impact**: ~18% of all allocations in our code

5. **Card loader allocations** - 269 calls, 24.58KB (src/loader/database_async.rs:88)
   - One-time cost during game setup
   - **Priority**: Low (not per-turn, but largest individual allocation)

#### Medium Priority

- [ ] **Vec reallocations** - Many small Vec allocations for temporary collections
  - Review `game_loop.rs:418` (player_ids collection)
  - Review `combat.rs:90,95` (attackers/blockers lists)
  - **Solution**: Return iterators instead of `Vec` where possible

- [ ] **Zone transfer operations** - Moving cards between zones (hand→battlefield→graveyard)
  - Potential temporary allocations during card movement
  - **Solution**: Audit and minimize intermediate allocations

- [ ] **Mana pool calculations** - ManaEngine operations during cost payment
  - Review for unnecessary cloning of mana costs
  - Seen in `game_loop.rs:106,277` (mana_cost.clone())

#### Low Priority

- [ ] **Consider arena allocation** for per-turn temporary objects
- [ ] **Object pooling** for frequently created/destroyed effects
- [ ] **Investigate intern patterns** for card names and string literals
- [ ] **Compile-time feature flag** to disable verbose logging in release builds

### Key Insights from Profiling

**Logging is the #1 allocation source** (>70% of allocations in our code):
- String formatting via `format!()` dominates allocation count
- Options to address:
  1. Use `tracing` crate with zero-cost disabled spans
  2. Implement string interning for repeated messages
  3. Add compile-time feature flag to disable logging
  4. Use `Cow<'static, str>` for common log messages
  5. Pre-allocate string buffers and reuse them

**Good news**: Most allocations are small (bytes to KB range), not large collections. The code isn't doing pathological things like cloning entire game states.

### Optimization Wins

Track completed optimizations and their measured impact here:

#### 1. Conditional Compilation of Logging (mtg-6) - commit#165

**Problem**: String formatting in logging was the #1 allocation hotspot (70%+ of allocations):
- 77,378 calls in Combat.clear() logging
- 45,274 calls in draw card logging
- 43,437 calls in discard logging
- Every `format!()` macro allocates even when verbosity level is `Silent`

**Solution**: Implemented compile-time feature flag `verbose-logging`:
- Created `log_if_verbose!()` macro that conditionally compiles logging code
- When feature is disabled, logging is completely eliminated at compile time (zero cost)
- Enabled by default for backward compatibility

**Results**:
- With feature enabled (default): Behavior unchanged, ~607-633 games/sec
- With feature disabled (`--no-default-features`): Eliminates ALL logging allocations
- Sets pattern for future zero-cost conditional features

**Usage**:
```bash
# Performance benchmarks without logging:
cargo bench --no-default-features
```

**Files modified**:
- `Cargo.toml`: Added `verbose-logging` feature (default)
- `src/game/game_loop.rs`: Added macro and replaced 5 high-frequency logging calls

## Future Directions

### Rewind/Undo System

The PROJECT_VISION.md describes plans for an undo log system to enable efficient game tree search. This will be critical for AI development and should be designed with zero-copy principles:

- Use unboxed enum for `GameAction` variants
- Store in contiguous `Vec` or arena
- Compile-time flag to disable undo logging for pure replay benchmarks
- Minimize action granularity (what's the minimum state change needed?)

### Compile-Time Flags for Profiling

Consider feature flags for different optimization profiles:
- `zero-copy-strict`: Enforce at compile time (return impl Iterator, deny clone in certain modules)
- `undo-logging`: Enable/disable undo log overhead
- `debug-allocations`: Track allocation sites for profiling

## References

- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Rust Performance Pitfalls](https://llogiq.github.io/2017/06/01/perf-pitfalls.html)
- [Arenas in Rust](https://manishearth.github.io/blog/2021/03/15/arenas-in-rust/)
- [Zero-Copy in Rust (CoinsBench)](https://coinsbench.com/zero-copy-in-rust-challenges-and-solutions-c0d38a6468e9)

## Profiling Tools

- **cargo bench**: Built-in benchmark harness (benches/game_benchmark.rs)
- **heaptrack**: Heap profiling for allocation tracking (`make heapprofile`)
- **flamegraph**: CPU profiling for hotspot identification (`make profile`)
- **valgrind/cachegrind**: Cache behavior analysis (manual setup)

---

**Note**: This is a living document. Update it as we discover new patterns, complete optimizations, or identify new bottlenecks through profiling.
