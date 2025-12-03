#!/bin/bash
# Benchmark comparison between Reukocyte and RuboCop
# Usage: ./scripts/benchmark_comparison.sh

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Reukocyte vs RuboCop Benchmark ===${NC}\n"

# Build release version
echo -e "${YELLOW}Building release version...${NC}"
cargo build --release --quiet

# Generate test files of different sizes
echo -e "${YELLOW}Generating test files...${NC}"

generate_ruby_file() {
    local lines=$1
    local file=$2

    echo "# frozen_string_literal: true" > "$file"
    echo "" >> "$file"
    echo "" >> "$file"  # Leading empty line (violation)
    echo "class Example" >> "$file"

    for i in $(seq 1 $lines); do
        if [ $((i % 10)) -eq 0 ]; then
            # Trailing whitespace
            echo "  def method_${i}  " >> "$file"
        elif [ $((i % 15)) -eq 0 ]; then
            # Tab indentation
            printf "\tdef method_${i}\n" >> "$file"
        else
            echo "  def method_${i}" >> "$file"
        fi
        echo "    @value = 42" >> "$file"
        echo "  end" >> "$file"

        if [ $((i % 20)) -eq 0 ]; then
            echo "" >> "$file"
            echo "" >> "$file"  # Extra empty lines (violation)
        fi
        echo "" >> "$file"
    done

    echo "end" >> "$file"
    echo "" >> "$file"
    echo "" >> "$file"  # Trailing empty lines (violation)
}

# Generate files
mkdir -p tmp/bench
generate_ruby_file 100 tmp/bench/small.rb
generate_ruby_file 1000 tmp/bench/medium.rb
generate_ruby_file 5000 tmp/bench/large.rb

echo -e "\n${BLUE}File sizes:${NC}"
ls -lh tmp/bench/*.rb | awk '{print "  " $9 ": " $5}'

# Layout rules to test
LAYOUT_RULES="Layout/TrailingWhitespace,Layout/EmptyLines,Layout/LeadingEmptyLines,Layout/IndentationStyle"

echo -e "\n${BLUE}=== Small file (100 methods, ~4KB) ===${NC}"
echo -e "${GREEN}Reukocyte:${NC}"
time target/release/rueko tmp/bench/small.rb 2>/dev/null | tail -1
echo -e "${GREEN}RuboCop:${NC}"
time rubocop tmp/bench/small.rb --only "$LAYOUT_RULES" -f simple 2>/dev/null | tail -1

echo -e "\n${BLUE}=== Medium file (1000 methods, ~40KB) ===${NC}"
echo -e "${GREEN}Reukocyte:${NC}"
time target/release/rueko tmp/bench/medium.rb 2>/dev/null | tail -1
echo -e "${GREEN}RuboCop:${NC}"
time rubocop tmp/bench/medium.rb --only "$LAYOUT_RULES" -f simple 2>/dev/null | tail -1

echo -e "\n${BLUE}=== Large file (5000 methods, ~200KB) ===${NC}"
echo -e "${GREEN}Reukocyte:${NC}"
time target/release/rueko tmp/bench/large.rb 2>/dev/null | tail -1
echo -e "${GREEN}RuboCop (server mode):${NC}"
time rubocop tmp/bench/large.rb --only "$LAYOUT_RULES" -f simple 2>/dev/null | tail -1

echo -e "\n${BLUE}=== Detailed timing (100 runs on small file) ===${NC}"
echo -e "${GREEN}Reukocyte:${NC}"
hyperfind_or_time() {
    if command -v hyperfine &> /dev/null; then
        hyperfine --warmup 3 --runs 100 "target/release/rueko tmp/bench/small.rb" 2>/dev/null
    else
        echo "  (install hyperfine for detailed timing)"
        for i in {1..10}; do
            time target/release/rueko tmp/bench/small.rb 2>/dev/null
        done 2>&1 | grep real | awk '{sum+=$2; n++} END {print "  Average: " sum/n "s"}'
    fi
}
hyperfind_or_time

echo -e "\n${BLUE}=== Summary ===${NC}"
echo "Reukocyte implements these Layout rules:"
echo "  - Layout/TrailingWhitespace"
echo "  - Layout/TrailingEmptyLines"
echo "  - Layout/LeadingEmptyLines"
echo "  - Layout/EmptyLines"
echo "  - Layout/IndentationStyle"
echo ""
echo "Plus Lint/Debugger for AST-based checking."
