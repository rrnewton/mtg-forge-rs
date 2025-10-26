# Scripts Directory

This directory contains utility scripts for development, testing, and performance tracking.

## Benchmark Performance Tracking

The benchmark tracking system automatically records performance metrics over time, enabling regression detection and performance analysis.

### Quick Start

```bash
# Run benchmarks and append results to history
./scripts/run_benchmark

# Run specific benchmark only (faster)
./scripts/run_benchmark snapshot

# Backfill historical data (samples 10 recent commits)
./scripts/backfill_history --count 10 --benchmark snapshot

# Generate performance plots
./scripts/plot_performance

# Show plots interactively (requires X11/display)
./scripts/plot_performance --show
```

### Scripts

#### `run_benchmark`

Runs benchmarks and appends results to `experiment_results/perf_history.csv`.

**Usage:**
```bash
./scripts/run_benchmark [benchmark_name]
```

**Examples:**
```bash
# Run all benchmarks (fresh, snapshot, rewind, logging variants)
./scripts/run_benchmark

# Run only snapshot benchmark (fastest, recommended for tracking)
./scripts/run_benchmark snapshot

# Run only fresh benchmark
./scripts/run_benchmark fresh
```

**What it does:**
1. Collects git metadata (commit hash, depth, branch, dirty status)
2. Runs `cargo bench --bench game_benchmark [filter]`
3. Parses aggregated metrics from benchmark output
4. Appends CSV row with timestamp and all metrics
5. Creates CSV header if file doesn't exist

**Output:** Updates `experiment_results/perf_history.csv`

#### `backfill_history`

Backfills performance history by checking out and benchmarking historical commits.

**Usage:**
```bash
./scripts/backfill_history [options]
```

**Options:**
- `--start COMMIT` - Start from this commit (default: oldest)
- `--end COMMIT` - End at this commit (default: HEAD)
- `--count N` - Number of commits to sample (default: 10)
- `--benchmark NAME` - Specific benchmark to run (default: snapshot)
- `--skip-existing` - Skip commits already in history

**Examples:**
```bash
# Sample 10 commits from entire history
./scripts/backfill_history --count 10

# Sample last 20 commits
./scripts/backfill_history --start HEAD~20 --end HEAD --count 10

# Backfill with fresh benchmark instead of snapshot
./scripts/backfill_history --benchmark fresh --count 5

# Skip commits already benchmarked
./scripts/backfill_history --skip-existing --count 20
```

**What it does:**
1. Saves current git state
2. Samples commits evenly across specified range
3. For each commit:
   - Checks out the commit
   - Verifies benchmark file and resources exist
   - Builds and runs benchmark with timeout (5 min)
   - Records results with that commit's metadata
   - Returns to current state
4. Restores original git state when done

**Safety:**
- Checks for uncommitted changes before starting
- Always restores original state on exit (even on error)
- Uses 5-minute timeout per benchmark to prevent hangs
- Skips commits that fail to build or benchmark

**Performance:** Benchmarking takes ~10-30 seconds per commit depending on benchmark type. Budget ~5 minutes for 10 commits with snapshot benchmark.

#### `plot_performance`

Generates performance visualization plots from history CSV.

**Requirements:** Python 3 with pandas and matplotlib:
```bash
pip install pandas matplotlib
```

**Usage:**
```bash
./scripts/plot_performance [options]
```

**Options:**
- `--input FILE` - Input CSV file (default: experiment_results/perf_history.csv)
- `--output DIR` - Output directory (default: experiment_results/plots)
- `--benchmark NAME` - Filter to specific benchmark
- `--metric METRIC` - Plot specific metric only
- `--format FORMAT` - Output format: png, svg, pdf (default: png)
- `--show` - Show plots interactively

**Examples:**
```bash
# Generate all plots as PNG
./scripts/plot_performance

# Generate SVG plots for fresh benchmark only
./scripts/plot_performance --benchmark fresh --format svg

# Plot only games_per_sec metric
./scripts/plot_performance --metric games_per_sec

# Show plots in GUI (requires display)
./scripts/plot_performance --show
```

**What it does:**
1. Loads performance history CSV
2. Detects performance regressions (>15% drop from best)
3. Generates time-series plots for key metrics:
   - Games per second (throughput)
   - Actions per second (throughput)
   - Average game duration
   - Memory: bytes per game, bytes per turn
   - Game complexity: actions per turn
   - Game length: average turns
4. Marks regressions with red X markers
5. Annotates worst regression with percentage drop
6. Saves plots to output directory

**Output:** PNG/SVG/PDF files in `experiment_results/plots/`

### Performance History CSV Format

The `perf_history.csv` file contains one row per benchmark run with the following columns:

#### Metadata
- `timestamp` - ISO 8601 UTC timestamp (e.g., "2025-10-26T14:30:00Z")
- `git_commit` - Short git commit hash (8 chars)
- `git_depth` - Number of commits in history (for chronological ordering)
- `git_branch` - Git branch name
- `git_dirty` - "_dirty" if uncommitted changes, empty otherwise

#### Benchmark Info
- `benchmark_name` - Benchmark type: "fresh", "snapshot", "rewind", "fresh_logging", "fresh_stdout_logging"
- `seed` - Random seed used (usually 42)
- `num_games` - Number of game iterations run

#### Aggregate Metrics
- `total_turns` - Total turns across all games
- `total_actions` - Total actions (undo log entries) across all games
- `total_duration_ms` - Total duration in milliseconds

#### Average Metrics
- `avg_turns_per_game` - Average turns per game
- `avg_actions_per_game` - Average actions per game
- `avg_duration_ms_per_game` - Average duration per game (ms)

#### Throughput Metrics
- `games_per_sec` - Games completed per second
- `actions_per_sec` - Actions per second
- `turns_per_sec` - Turns per second
- `actions_per_turn` - Average actions per turn (game complexity)

#### Memory Metrics
- `total_bytes_allocated` - Total bytes allocated across all games
- `total_bytes_deallocated` - Total bytes deallocated
- `net_bytes` - Net allocation (allocated - deallocated)
- `avg_bytes_per_game` - Average bytes allocated per game
- `bytes_per_turn` - Bytes allocated per turn
- `bytes_per_sec` - Allocation rate (bytes per second)

### Recommended Workflow

#### Regular Development

After making changes, run benchmarks to track performance:

```bash
# Run fast snapshot benchmark only
./scripts/run_benchmark snapshot

# Check latest results
tail -n 2 experiment_results/perf_history.csv

# Generate updated plots
./scripts/plot_performance
```

#### Before Release

Generate comprehensive performance analysis:

```bash
# Run all benchmarks
./scripts/run_benchmark

# Backfill last 20 commits for historical context
./scripts/backfill_history --start HEAD~20 --count 20 --benchmark snapshot

# Generate plots with regression detection
./scripts/plot_performance

# Review plots in experiment_results/plots/
```

#### Investigating Regressions

If plots show a regression:

1. Identify the commit from the plot's x-axis (git depth) or CSV
2. Use git to examine changes:
   ```bash
   # Find commit by depth
   git log --oneline | head -n <depth> | tail -n 1

   # Compare with previous commit
   git diff <commit>~1 <commit>

   # Check commit message and changes
   git show <commit>
   ```
3. Profile the regression:
   ```bash
   git checkout <commit>
   make profile  # Generates flamegraph
   make heapprofile  # Generates allocation profile
   ```

### Benchmark Types

Different benchmark modes measure different aspects:

- **fresh** - Allocates new game each iteration. Measures full initialization cost.
- **snapshot** - Uses `Clone` to copy initial state. Measures clone overhead and normal gameplay.
- **rewind** - Uses undo log to rewind. Measures undo/tree search performance.
- **fresh_logging** - Fresh mode with in-memory log capture. Measures logging allocation overhead.
- **fresh_stdout_logging** - Fresh mode with stdout logging. Measures reusable buffer optimization.

**Recommendation:** Use `snapshot` benchmark for routine tracking as it's fastest and most representative of tree search workloads (MCTS, minimax).

### Timestamp Format

For consistency with project conventions, when documenting results in issues or commits, use the full timestamp format:

```
YYYY-MM-DD_#DEPTH(shortcommit)
```

Example: `2025-10-26_#161(224384cd)`

This format includes:
- Date: `2025-10-26`
- Git depth: `#161` (from `git rev-list --count HEAD`)
- Commit: `224384cd` (short hash)

You can get the current timestamp with:
```bash
echo "$(date +%Y-%m-%d)_#$(git rev-list --count HEAD)($(git rev-parse --short HEAD))"
```

## Other Scripts

### `run_examples.sh`

Runs all example binaries to verify they execute successfully.

**Usage:**
```bash
./scripts/run_examples.sh
```

Used by `make validate` and CI workflows.

### `validate.sh`

Runs comprehensive validation with caching.

**Usage:**
```bash
./scripts/validate.sh [--force] [--sequential]
```

**Options:**
- `--force` - Skip cache, always run validation
- `--sequential` - Run steps sequentially (fail fast)

Normally invoked via `make validate`.

### `analyze_heapprofile.sh`

Analyzes heap profiling results from `make heapprofile`.

**Usage:**
```bash
make heapprofile  # Generates heaptrack data
# Script automatically runs to show top allocation sites
```

## Adding New Scripts

When adding new scripts:

1. Make them executable: `chmod +x scripts/script_name`
2. Add shebang line: `#!/bin/bash` or `#!/usr/bin/env python3`
3. Include usage documentation in comments
4. Update this README with description and examples
5. Add Makefile targets if appropriate
