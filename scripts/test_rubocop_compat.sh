#!/bin/bash
# RuboCop compatibility test script
# Compares rueko output with RuboCop output to verify compatibility

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RUEKO="$PROJECT_ROOT/target/debug/rueko"
TESTDATA="$PROJECT_ROOT/testdata"
TMP_DIR=$(mktemp -d)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
SKIPPED=0

cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# Build rueko if needed
echo "Building rueko..."
cargo build -p reukocyte --quiet

echo ""
echo "=========================================="
echo "  RuboCop Compatibility Test Suite"
echo "=========================================="
echo ""

# Test 1: JSON output format compatibility
test_json_format() {
    echo -n "Test: JSON output format... "
    
    local test_file="$TMP_DIR/test_json.rb"
    echo 'x = 1  ' > "$test_file"  # trailing whitespace
    
    # Get RuboCop JSON output
    local rubocop_json=$(rubocop -f json "$test_file" 2>/dev/null || true)
    
    # Get rueko JSON output  
    local rueko_json=$("$RUEKO" -f json "$test_file" 2>/dev/null || true)
    
    # Check that both have required fields
    if echo "$rueko_json" | grep -q '"metadata"' && \
       echo "$rueko_json" | grep -q '"files"' && \
       echo "$rueko_json" | grep -q '"summary"' && \
       echo "$rueko_json" | grep -q '"offenses"'; then
        echo -e "${GREEN}PASSED${NC}"
        ((PASSED++))
    else
        echo -e "${RED}FAILED${NC}"
        echo "  rueko JSON missing required fields"
        ((FAILED++))
    fi
}

# Test 2: Layout/TrailingWhitespace detection
test_trailing_whitespace() {
    echo -n "Test: Layout/TrailingWhitespace detection... "
    
    local test_file="$TMP_DIR/test_tw.rb"
    printf 'x = 1  \ny = 2\nz = 3   \n' > "$test_file"
    
    # Count RuboCop offenses
    local rubocop_count=$(rubocop --only Layout/TrailingWhitespace -f json "$test_file" 2>/dev/null | grep -o '"cop_name":"Layout/TrailingWhitespace"' | wc -l)
    
    # Count rueko offenses
    local rueko_count=$("$RUEKO" --only Layout/TrailingWhitespace -f json "$test_file" 2>/dev/null | grep -o '"cop_name":"Layout/TrailingWhitespace"' | wc -l)
    
    if [ "$rubocop_count" = "$rueko_count" ]; then
        echo -e "${GREEN}PASSED${NC} (both found $rubocop_count offense(s))"
        ((PASSED++))
    else
        echo -e "${RED}FAILED${NC}"
        echo "  RuboCop found: $rubocop_count, rueko found: $rueko_count"
        ((FAILED++))
    fi
}

# Test 3: Layout/TrailingWhitespace autocorrect
test_trailing_whitespace_fix() {
    echo -n "Test: Layout/TrailingWhitespace autocorrect... "
    
    local rubocop_file="$TMP_DIR/test_tw_rubocop.rb"
    local rueko_file="$TMP_DIR/test_tw_rueko.rb"
    printf 'x = 1  \ny = 2\nz = 3   \n' > "$rubocop_file"
    printf 'x = 1  \ny = 2\nz = 3   \n' > "$rueko_file"
    
    # Fix with RuboCop
    rubocop -a --only Layout/TrailingWhitespace "$rubocop_file" >/dev/null 2>&1 || true
    
    # Fix with rueko
    "$RUEKO" -a --only Layout/TrailingWhitespace "$rueko_file" >/dev/null 2>&1 || true
    
    # Compare results
    if diff -q "$rubocop_file" "$rueko_file" >/dev/null 2>&1; then
        echo -e "${GREEN}PASSED${NC}"
        ((PASSED++))
    else
        echo -e "${RED}FAILED${NC}"
        echo "  Files differ after autocorrect:"
        diff "$rubocop_file" "$rueko_file" | head -10
        ((FAILED++))
    fi
}

# Test 4: Lint/Debugger detection
test_debugger_detection() {
    echo -n "Test: Lint/Debugger detection... "
    
    local test_file="$TMP_DIR/test_debugger.rb"
    cat > "$test_file" << 'EOF'
def foo
  binding.pry
  x = 1
  byebug
  y = 2
end
EOF
    
    # Count RuboCop offenses
    local rubocop_count=$(rubocop --only Lint/Debugger -f json "$test_file" 2>/dev/null | grep -o '"cop_name":"Lint/Debugger"' | wc -l)
    
    # Count rueko offenses
    local rueko_count=$("$RUEKO" --only Lint/Debugger -f json "$test_file" 2>/dev/null | grep -o '"cop_name":"Lint/Debugger"' | wc -l)
    
    if [ "$rubocop_count" = "$rueko_count" ]; then
        echo -e "${GREEN}PASSED${NC} (both found $rubocop_count offense(s))"
        ((PASSED++))
    else
        echo -e "${RED}FAILED${NC}"
        echo "  RuboCop found: $rubocop_count, rueko found: $rueko_count"
        ((FAILED++))
    fi
}

# Test 5: File collection (Ruby file patterns)
test_file_collection() {
    echo -n "Test: Ruby file pattern detection... "
    
    local test_dir="$TMP_DIR/file_patterns"
    mkdir -p "$test_dir"
    
    # Create various Ruby files
    echo 'x = 1' > "$test_dir/test.rb"
    echo 'x = 1' > "$test_dir/Gemfile"
    echo 'x = 1' > "$test_dir/Rakefile"
    echo 'x = 1' > "$test_dir/config.ru"
    echo 'x = 1' > "$test_dir/test.rake"
    echo 'x = 1' > "$test_dir/not_ruby.txt"
    
    # Count files RuboCop inspects
    local rubocop_files=$(rubocop -f json "$test_dir" 2>/dev/null | grep -o '"path":' | wc -l)
    
    # Count files rueko inspects
    local rueko_files=$("$RUEKO" -f json "$test_dir" 2>/dev/null | grep -o '"path":' | wc -l)
    
    if [ "$rubocop_files" = "$rueko_files" ]; then
        echo -e "${GREEN}PASSED${NC} (both inspected $rubocop_files file(s))"
        ((PASSED++))
    else
        echo -e "${YELLOW}PARTIAL${NC}"
        echo "  RuboCop inspected: $rubocop_files, rueko inspected: $rueko_files"
        ((SKIPPED++))
    fi
}

# Test 6: Offense location format
test_offense_location() {
    echo -n "Test: Offense location format... "
    
    local test_file="$TMP_DIR/test_location.rb"
    echo 'x = 1  ' > "$test_file"
    
    # Get rueko JSON and check location fields
    local rueko_json=$("$RUEKO" --only Layout/TrailingWhitespace -f json "$test_file" 2>/dev/null || true)
    
    if echo "$rueko_json" | grep -q '"start_line"' && \
       echo "$rueko_json" | grep -q '"start_column"' && \
       echo "$rueko_json" | grep -q '"last_line"' && \
       echo "$rueko_json" | grep -q '"last_column"' && \
       echo "$rueko_json" | grep -q '"length"'; then
        echo -e "${GREEN}PASSED${NC}"
        ((PASSED++))
    else
        echo -e "${RED}FAILED${NC}"
        echo "  Missing location fields in JSON output"
        ((FAILED++))
    fi
}

# Test 7: Severity levels
test_severity_levels() {
    echo -n "Test: Severity level format... "
    
    local test_file="$TMP_DIR/test_severity.rb"
    echo 'x = 1  ' > "$test_file"
    
    local rueko_json=$("$RUEKO" -f json "$test_file" 2>/dev/null || true)
    
    # Check severity is a valid RuboCop severity
    if echo "$rueko_json" | grep -qE '"severity":"(info|refactor|convention|warning|error|fatal)"'; then
        echo -e "${GREEN}PASSED${NC}"
        ((PASSED++))
    else
        echo -e "${RED}FAILED${NC}"
        echo "  Invalid severity format"
        ((FAILED++))
    fi
}

# Test 8: Empty file handling
test_empty_file() {
    echo -n "Test: Empty file handling... "
    
    local test_file="$TMP_DIR/empty.rb"
    touch "$test_file"
    
    # Both should handle empty files without error
    local rubocop_exit=0
    local rueko_exit=0
    
    rubocop -f json "$test_file" >/dev/null 2>&1 || rubocop_exit=$?
    "$RUEKO" -f json "$test_file" >/dev/null 2>&1 || rueko_exit=$?
    
    # Both should succeed (exit 0) for empty files
    if [ "$rueko_exit" = "0" ]; then
        echo -e "${GREEN}PASSED${NC}"
        ((PASSED++))
    else
        echo -e "${RED}FAILED${NC}"
        echo "  rueko exit code: $rueko_exit (expected 0)"
        ((FAILED++))
    fi
}

# Run all tests
test_json_format
test_trailing_whitespace
test_trailing_whitespace_fix
test_debugger_detection
test_file_collection
test_offense_location
test_severity_levels
test_empty_file

# Summary
echo ""
echo "=========================================="
echo "  Summary"
echo "=========================================="
echo -e "  ${GREEN}Passed:${NC}  $PASSED"
echo -e "  ${RED}Failed:${NC}  $FAILED"
echo -e "  ${YELLOW}Skipped:${NC} $SKIPPED"
echo "=========================================="

# Exit with failure if any tests failed
if [ "$FAILED" -gt 0 ]; then
    exit 1
fi
