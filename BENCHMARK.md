# Benchmark Results

## Environment
- **Date**: December 2024
- **Platform**: Linux (Debian GNU/Linux 13)
- **Rust**: Edition 2024
- **Tool**: hyperfine

## Summary

| File Size | Methods | Reukocyte | RuboCop | Speedup |
|-----------|---------|-----------|---------|---------|
| 4 KB | 100 | ~1 ms | ~266 ms | **~260x** |
| 40 KB | 1,000 | **1.9 ms** | 228 ms | **~120x** |
| 200 KB | 5,000 | **4.8 ms** | 243 ms | **~50x** |

## Implemented Rules

### Layout Rules (Phase 1 Complete)
- [x] Layout/TrailingWhitespace - Trailing whitespace detection
- [x] Layout/TrailingEmptyLines - Final newline and trailing blank lines
- [x] Layout/LeadingEmptyLines - Leading blank lines at file start
- [x] Layout/EmptyLines - Consecutive blank lines (2+)
- [x] Layout/IndentationStyle - Tab vs space indentation

### Lint Rules
- [x] Lint/Debugger - Debug statement detection

## Detailed Results

### Medium File (40KB, 1000 methods)

```
Benchmark 1: target/release/reuko tmp/bench/medium.rb
  Time (mean ± σ):       1.9 ms ±   0.3 ms
  Range (min … max):     1.6 ms …   3.0 ms

Benchmark 2: rubocop --only Layout/TrailingWhitespace,...
  Time (mean ± σ):     228.1 ms ±  10.7 ms
  Range (min … max):   219.4 ms … 284.3 ms

Summary: Reukocyte ran 119.61 ± 22.27 times faster
```

### Large File (200KB, 5000 methods)

```
Benchmark 1: target/release/reuko tmp/bench/large.rb
  Time (mean ± σ):       4.8 ms ±   0.8 ms
  Range (min … max):     4.0 ms …   7.1 ms

Benchmark 2: rubocop --only Layout/...
  Time (mean ± σ):     242.6 ms ±   6.0 ms
  Range (min … max):   236.4 ms … 264.6 ms

Summary: Reukocyte ran 50.14 ± 8.65 times faster
```

## Throughput

Based on criterion benchmarks:

| File Size | Throughput |
|-----------|------------|
| Clean code | ~80-90 MiB/s |
| With violations | ~70-80 MiB/s |

## Goal vs Actual

| Metric | Goal | Actual | Status |
|--------|------|--------|--------|
| vs RuboCop (server) | 40x faster | 50-120x | ✅ Exceeded |
| CI speedup target | 20x | TBD | 🔄 In progress |

## Running Benchmarks

```bash
# Criterion benchmarks (internal)
cargo bench --bench layout

# Comparison with RuboCop
./scripts/benchmark_comparison.sh

# Manual hyperfine comparison
hyperfine --warmup 5 -i \
  'target/release/reuko FILE.rb' \
  'rubocop FILE.rb --only Layout/... -f quiet'
```
