#!/usr/bin/env ruby
# Simple benchmark comparing PreCop vs RuboCop
# Usage: ruby benchmark.rb

require 'benchmark'
require 'fileutils'

SCRIPT_DIR = File.dirname(__FILE__)
PROJECT_ROOT = File.dirname(SCRIPT_DIR)
TEST_FILES_DIR = File.join(SCRIPT_DIR, 'test_files')
PRECOP = File.join(PROJECT_ROOT, 'target', 'release', 'precop')

# RuboCop config for only Layout cops
RUBOCOP_CONFIG = File.join(SCRIPT_DIR, '.rubocop_bench.yml')
File.write(RUBOCOP_CONFIG, <<~YAML)
  AllCops:
    DisabledByDefault: true
    SuggestExtensions: false

  Layout/TrailingWhitespace:
    Enabled: true

  Layout/TrailingEmptyLines:
    Enabled: true
YAML

def run_precop(file)
  `#{PRECOP} "#{file}" -q 2>&1`
end

def run_rubocop(file)
  `rubocop -c "#{RUBOCOP_CONFIG}" "#{file}" --format quiet 2>&1`
end

puts "=" * 70
puts "PreCop vs RuboCop Benchmark"
puts "=" * 70
puts ""

# Warm up
puts "Warming up..."
run_precop(File.join(TEST_FILES_DIR, 'trailing_whitespace_small.rb'))
run_rubocop(File.join(TEST_FILES_DIR, 'trailing_whitespace_small.rb'))
puts ""

results = []

# Test different file sizes
%w[small medium large].each do |size|
  file = File.join(TEST_FILES_DIR, "trailing_whitespace_#{size}.rb")
  next unless File.exist?(file)

  file_size = File.size(file)

  precop_times = []
  rubocop_times = []

  # Run multiple iterations
  iterations = 5

  iterations.times do
    precop_times << Benchmark.realtime { run_precop(file) }
    rubocop_times << Benchmark.realtime { run_rubocop(file) }
  end

  precop_avg = (precop_times.sum / iterations * 1000).round(2)
  rubocop_avg = (rubocop_times.sum / iterations * 1000).round(2)
  speedup = (rubocop_avg / precop_avg).round(1)

  results << {
    name: "trailing_whitespace_#{size}.rb",
    size: file_size,
    precop: precop_avg,
    rubocop: rubocop_avg,
    speedup: speedup
  }
end

# Test many files
many_files_dir = File.join(TEST_FILES_DIR, 'many_files')
if Dir.exist?(many_files_dir)
  files = Dir.glob(File.join(many_files_dir, '*.rb'))
  files_arg = files.map { |f| %("#{f}") }.join(' ')

  precop_times = []
  rubocop_times = []

  5.times do
    precop_times << Benchmark.realtime { `#{PRECOP} #{files_arg} -q 2>&1` }
    rubocop_times << Benchmark.realtime { `rubocop -c "#{RUBOCOP_CONFIG}" #{files_arg} --format quiet 2>&1` }
  end

  precop_avg = (precop_times.sum / 5 * 1000).round(2)
  rubocop_avg = (rubocop_times.sum / 5 * 1000).round(2)
  speedup = (rubocop_avg / precop_avg).round(1)

  results << {
    name: "100 files (parallel)",
    size: files.sum { |f| File.size(f) },
    precop: precop_avg,
    rubocop: rubocop_avg,
    speedup: speedup
  }
end

# Print results
puts "Results (average of 5 runs):"
puts "-" * 70
printf "%-30s %10s %12s %12s %10s\n", "Test", "Size", "PreCop (ms)", "RuboCop (ms)", "Speedup"
puts "-" * 70

results.each do |r|
  size_str = r[:size] < 1024 ? "#{r[:size]} B" : "#{(r[:size] / 1024.0).round(1)} KB"
  printf "%-30s %10s %12.2f %12.2f %10.1fx\n",
         r[:name], size_str, r[:precop], r[:rubocop], r[:speedup]
end

puts "-" * 70
puts ""

# Cleanup
File.delete(RUBOCOP_CONFIG) if File.exist?(RUBOCOP_CONFIG)

avg_speedup = (results.sum { |r| r[:speedup] } / results.size).round(1)
puts "Average speedup: #{avg_speedup}x faster"
puts ""
puts "Note: RuboCop startup time dominates for small files."
puts "      The real benefit shows with many files (parallel processing)."
