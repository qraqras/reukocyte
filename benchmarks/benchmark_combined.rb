#!/usr/bin/env ruby
# Benchmark: PreCop + RuboCop (excluding Layout) vs RuboCop (all cops)
#
# Real-world scenario: Use PreCop for Layout cops, RuboCop for everything else
# This shows the actual time savings when combining both tools.

require 'benchmark'
require 'fileutils'

SCRIPT_DIR = File.dirname(__FILE__)
PROJECT_ROOT = File.dirname(SCRIPT_DIR)
TEST_FILES_DIR = File.join(SCRIPT_DIR, 'test_files')
PRECOP = File.join(PROJECT_ROOT, 'target', 'release', 'precop')

# Create temp directory for configs
TEMP_DIR = File.join(SCRIPT_DIR, 'temp_combo_bench')
FileUtils.mkdir_p(TEMP_DIR)
FileUtils.cp_r(TEST_FILES_DIR, File.join(TEMP_DIR, 'test_files'))
TEST_DIR = File.join(TEMP_DIR, 'test_files')

# Config 1: RuboCop with ALL cops (simulating typical usage)
RUBOCOP_ALL_CONFIG = File.join(TEMP_DIR, '.rubocop_all.yml')
File.write(RUBOCOP_ALL_CONFIG, <<~YAML)
  AllCops:
    SuggestExtensions: false
    NewCops: disable
YAML

# Config 2: RuboCop WITHOUT Layout cops (used with PreCop)
RUBOCOP_NO_LAYOUT_CONFIG = File.join(TEMP_DIR, '.rubocop_no_layout.yml')
File.write(RUBOCOP_NO_LAYOUT_CONFIG, <<~YAML)
  AllCops:
    SuggestExtensions: false
    NewCops: disable

  # Disable all Layout cops (PreCop handles these)
  Layout:
    Enabled: false
YAML

# Config 3: RuboCop with ONLY Layout cops (for comparison)
RUBOCOP_LAYOUT_ONLY_CONFIG = File.join(TEMP_DIR, '.rubocop_layout_only.yml')
File.write(RUBOCOP_LAYOUT_ONLY_CONFIG, <<~YAML)
  AllCops:
    DisabledByDefault: true
    SuggestExtensions: false
    NewCops: disable

  Layout:
    Enabled: true
YAML

def run_precop(files)
  files_arg = Array(files).map { |f| %("#{f}") }.join(' ')
  `#{PRECOP} #{files_arg} -q 2>&1`
end

def run_rubocop(config, files)
  files_arg = Array(files).map { |f| %("#{f}") }.join(' ')
  `rubocop -c "#{config}" #{files_arg} --format quiet 2>&1`
end

def stop_server
  `rubocop --stop-server 2>&1`
  sleep 0.2
end

puts "=" * 80
puts "Benchmark: PreCop + RuboCop vs RuboCop Only"
puts "=" * 80
puts ""
puts "Scenario: Typical CI/CD pipeline running all linting checks"
puts ""
puts "Configurations:"
puts "  1. RuboCop (all cops)         - Standard RuboCop usage"
puts "  2. RuboCop (Layout only)      - Just Layout cops"
puts "  3. RuboCop (no Layout)        - All cops except Layout"
puts "  4. PreCop + RuboCop (no Layout) - Combined approach"
puts ""

stop_server

rubocop_version = `rubocop --version`.strip
puts "RuboCop version: #{rubocop_version}"
puts ""

# Warm up
puts "Warming up..."
small_file = File.join(TEST_DIR, 'trailing_whitespace_small.rb')
run_precop(small_file)
run_rubocop(RUBOCOP_ALL_CONFIG, small_file)
run_rubocop(RUBOCOP_NO_LAYOUT_CONFIG, small_file)
puts ""

results = []
iterations = 3  # Fewer iterations since RuboCop all cops is slow

test_files = [
  { name: "small (1.8KB)", files: [File.join(TEST_DIR, 'trailing_whitespace_small.rb')] },
  { name: "medium (18.5KB)", files: [File.join(TEST_DIR, 'trailing_whitespace_medium.rb')] },
  { name: "large (96.6KB)", files: [File.join(TEST_DIR, 'trailing_whitespace_large.rb')] },
  { name: "100 files", files: Dir.glob(File.join(TEST_DIR, 'many_files', '*.rb')) }
]

test_files.each do |test|
  next if test[:files].empty? || !test[:files].all? { |f| File.exist?(f) }

  puts "Testing: #{test[:name]}..."

  precop_times = []
  rubocop_all_times = []
  rubocop_layout_times = []
  rubocop_no_layout_times = []

  iterations.times do
    # PreCop (Layout only)
    precop_times << Benchmark.realtime { run_precop(test[:files]) }

    # RuboCop with all cops
    rubocop_all_times << Benchmark.realtime { run_rubocop(RUBOCOP_ALL_CONFIG, test[:files]) }

    # RuboCop Layout only
    rubocop_layout_times << Benchmark.realtime { run_rubocop(RUBOCOP_LAYOUT_ONLY_CONFIG, test[:files]) }

    # RuboCop without Layout
    rubocop_no_layout_times << Benchmark.realtime { run_rubocop(RUBOCOP_NO_LAYOUT_CONFIG, test[:files]) }
  end

  precop_avg = (precop_times.sum / iterations * 1000).round(2)
  rubocop_all_avg = (rubocop_all_times.sum / iterations * 1000).round(2)
  rubocop_layout_avg = (rubocop_layout_times.sum / iterations * 1000).round(2)
  rubocop_no_layout_avg = (rubocop_no_layout_times.sum / iterations * 1000).round(2)

  # Combined time: PreCop + RuboCop (no Layout)
  combined_avg = precop_avg + rubocop_no_layout_avg

  results << {
    name: test[:name],
    precop: precop_avg,
    rubocop_all: rubocop_all_avg,
    rubocop_layout: rubocop_layout_avg,
    rubocop_no_layout: rubocop_no_layout_avg,
    combined: combined_avg
  }
end

puts ""
puts "=" * 80
puts "RESULTS (average of #{iterations} runs)"
puts "=" * 80
puts ""

# Table 1: Raw times
puts "Execution Times:"
puts "-" * 80
printf "%-15s %12s %12s %12s %12s %12s\n",
       "Test", "PreCop", "RC All", "RC Layout", "RC NoLayout", "PreCop+RC"
puts "-" * 80

results.each do |r|
  printf "%-15s %10.1f ms %10.1f ms %10.1f ms %10.1f ms %10.1f ms\n",
         r[:name], r[:precop], r[:rubocop_all], r[:rubocop_layout], r[:rubocop_no_layout], r[:combined]
end

puts ""
puts "Legend:"
puts "  PreCop      = PreCop (Layout cops only)"
puts "  RC All      = RuboCop with all cops enabled"
puts "  RC Layout   = RuboCop with only Layout cops"
puts "  RC NoLayout = RuboCop with Layout cops disabled"
puts "  PreCop+RC   = PreCop + RuboCop (no Layout) combined"
puts ""

# Table 2: Comparison
puts "=" * 80
puts "COMPARISON: PreCop+RuboCop vs RuboCop Only"
puts "=" * 80
puts ""
puts "-" * 80
printf "%-15s %14s %14s %14s %12s\n",
       "Test", "RuboCop All", "PreCop+RC", "Time Saved", "Speedup"
puts "-" * 80

total_rubocop_all = 0
total_combined = 0

results.each do |r|
  time_saved = r[:rubocop_all] - r[:combined]
  speedup = r[:rubocop_all] / r[:combined]

  total_rubocop_all += r[:rubocop_all]
  total_combined += r[:combined]

  printf "%-15s %12.1f ms %12.1f ms %12.1f ms %11.2fx\n",
         r[:name], r[:rubocop_all], r[:combined], time_saved, speedup
end

puts "-" * 80

total_saved = total_rubocop_all - total_combined
total_speedup = total_rubocop_all / total_combined
printf "%-15s %12.1f ms %12.1f ms %12.1f ms %11.2fx\n",
       "TOTAL", total_rubocop_all, total_combined, total_saved, total_speedup

puts ""

# Table 3: Layout cop comparison specifically
puts "=" * 80
puts "LAYOUT COP COMPARISON"
puts "=" * 80
puts ""
puts "-" * 80
printf "%-15s %14s %14s %12s\n",
       "Test", "RuboCop Layout", "PreCop", "Speedup"
puts "-" * 80

results.each do |r|
  speedup = r[:rubocop_layout] / r[:precop]
  printf "%-15s %12.1f ms %12.1f ms %11.1fx\n",
         r[:name], r[:rubocop_layout], r[:precop], speedup
end

puts ""

# Summary
puts "=" * 80
puts "SUMMARY"
puts "=" * 80
puts ""

avg_layout_speedup = (results.sum { |r| r[:rubocop_layout] / r[:precop] } / results.size).round(1)
avg_total_speedup = (total_rubocop_all / total_combined).round(2)
pct_saved = ((total_saved / total_rubocop_all) * 100).round(1)

puts "Layout cops only:"
puts "  PreCop is #{avg_layout_speedup}x faster than RuboCop for Layout cops"
puts ""
puts "Full linting pipeline (all cops):"
puts "  RuboCop only:     #{total_rubocop_all.round(0)} ms"
puts "  PreCop + RuboCop: #{total_combined.round(0)} ms"
puts "  Time saved:       #{total_saved.round(0)} ms (#{pct_saved}%)"
puts "  Overall speedup:  #{avg_total_speedup}x"
puts ""

# Real-world impact
puts "=" * 80
puts "REAL-WORLD IMPACT"
puts "=" * 80
puts ""

puts "For a typical CI run on 100 files:"
r = results.find { |x| x[:name] == "100 files" }
if r
  daily_runs = 50  # PRs per day
  monthly_runs = daily_runs * 22  # working days

  time_saved_per_run = r[:rubocop_all] - r[:combined]
  daily_saved = (time_saved_per_run * daily_runs / 1000).round(1)
  monthly_saved = (time_saved_per_run * monthly_runs / 1000 / 60).round(1)

  puts "  Per run saved:    #{time_saved_per_run.round(0)} ms"
  puts "  Daily saved:      #{daily_saved} seconds (#{daily_runs} runs/day)"
  puts "  Monthly saved:    #{monthly_saved} minutes"
  puts ""
  puts "For larger projects (1000+ files), multiply savings by 10x+"
end

puts ""

# Cleanup
FileUtils.rm_rf(TEMP_DIR)
puts "Cleanup complete."
