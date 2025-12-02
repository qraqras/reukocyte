#!/bin/bash
# Benchmark script comparing PreCop vs RuboCop

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TEST_FILES_DIR="$SCRIPT_DIR/test_files"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== PreCop vs RuboCop Benchmark ===${NC}"
echo ""

# Generate test files if not exist
if [ ! -d "$TEST_FILES_DIR" ]; then
    echo "Generating test files..."
    ruby "$SCRIPT_DIR/generate_test_files.rb"
    echo ""
fi

# Build PreCop in release mode
echo -e "${YELLOW}Building PreCop (release mode)...${NC}"
cd "$PROJECT_ROOT"
cargo build --release 2>/dev/null
PRECOP="$PROJECT_ROOT/target/release/precop"
echo ""

# Create RuboCop config for only Layout cops
RUBOCOP_CONFIG="$SCRIPT_DIR/.rubocop_bench.yml"
cat > "$RUBOCOP_CONFIG" << 'EOF'
AllCops:
  DisabledByDefault: true
  SuggestExtensions: false

Layout/TrailingWhitespace:
  Enabled: true

Layout/TrailingEmptyLines:
  Enabled: true
EOF

# Benchmark function
benchmark() {
    local name="$1"
    local cmd="$2"
    local file="$3"

    # Warm up
    eval "$cmd" > /dev/null 2>&1 || true

    # Run 5 times and get average
    local total=0
    local runs=5

    for i in $(seq 1 $runs); do
        local start=$(date +%s%N)
        eval "$cmd" > /dev/null 2>&1 || true
        local end=$(date +%s%N)
        local elapsed=$(( (end - start) / 1000000 ))
        total=$((total + elapsed))
    done

    local avg=$((total / runs))
    echo "$avg"
}

# Print results
print_result() {
    local test_name="$1"
    local precop_time="$2"
    local rubocop_time="$3"

    local speedup=$(echo "scale=1; $rubocop_time / $precop_time" | bc 2>/dev/null || echo "N/A")

    printf "%-35s %10s ms  %10s ms  %10sx\n" "$test_name" "$precop_time" "$rubocop_time" "$speedup"
}

echo -e "${GREEN}Test File                           PreCop       RuboCop       Speedup${NC}"
echo "------------------------------------------------------------------------"

# Test single files of different sizes
for size in small medium large; do
    for type in trailing_whitespace; do
        file="$TEST_FILES_DIR/${type}_${size}.rb"
        if [ -f "$file" ]; then
            precop_time=$(benchmark "precop" "$PRECOP \"$file\"" "$file")
            rubocop_time=$(benchmark "rubocop" "rubocop -c \"$RUBOCOP_CONFIG\" \"$file\"" "$file")
            print_result "${type}_${size}.rb" "$precop_time" "$rubocop_time"
        fi
    done
done

echo ""

# Test many small files (parallel processing test)
MANY_FILES="$TEST_FILES_DIR/many_files/*.rb"
if [ -d "$TEST_FILES_DIR/many_files" ]; then
    echo -e "${YELLOW}Testing 100 small files (parallel processing):${NC}"

    precop_time=$(benchmark "precop_many" "$PRECOP $MANY_FILES" "many")
    rubocop_time=$(benchmark "rubocop_many" "rubocop -c \"$RUBOCOP_CONFIG\" $MANY_FILES" "many")
    print_result "100 small files" "$precop_time" "$rubocop_time"
fi

echo ""
echo -e "${BLUE}=== Benchmark Complete ===${NC}"

# Cleanup
rm -f "$RUBOCOP_CONFIG"
