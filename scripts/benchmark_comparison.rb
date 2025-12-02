#!/usr/bin/env ruby
# frozen_string_literal: true

# Benchmark script comparing RuboCop vs Reukocyte
# Usage: ruby scripts/benchmark_comparison.rb

require 'benchmark'
require 'tempfile'
require 'fileutils'

# Generate test Ruby files of various sizes
def generate_ruby_file(lines, with_violations: false)
  content = []
  content << "# frozen_string_literal: true"
  content << ""
  content << "class Example"

  lines.times do |i|
    if with_violations && i % 10 == 0
      content << "  def method_#{i}  " # trailing whitespace
    else
      content << "  def method_#{i}"
    end
    content << "    @value = 42"
    content << "  end"
    content << ""
  end

  content << "end"
  content.join("\n")
end

def generate_debugger_file(methods, with_debuggers: false)
  content = []
  content << "# frozen_string_literal: true"
  content << ""
  content << "class Example"

  methods.times do |i|
    content << "  def method_#{i}"
    if with_debuggers && i % 10 == 0
      content << "    binding.pry"
    end
    content << "    @value = calculate_something"
    content << "    process(@value)"
    content << "  end"
    content << ""
  end

  content << "end"
  content.join("\n")
end

def run_rubocop(file_path, cops:)
  cop_list = cops.join(',')
  output = `rubocop --only #{cop_list} --format simple #{file_path} 2>&1`
  $?.exitstatus
end

def run_reukocyte(file_path, type:)
  # For now, we'll use cargo run (later can be optimized with pre-built binary)
  if type == :layout
    `cargo run -q --release -p reukocyte_layout -- #{file_path} 2>&1`
  else
    `cargo run -q --release -p reukocyte_lint -- #{file_path} 2>&1`
  end
  $?.exitstatus
end

ITERATIONS = 5
WARMUP = 2

puts "=" * 70
puts "RuboCop vs Reukocyte Benchmark Comparison"
puts "=" * 70
puts

# Create temp directory for test files
temp_dir = Dir.mktmpdir("reukocyte_bench")

begin
  # Test configurations
  tests = [
    { name: "Layout/TrailingWhitespace", lines: 100, violations: false },
    { name: "Layout/TrailingWhitespace", lines: 100, violations: true },
    { name: "Layout/TrailingWhitespace", lines: 500, violations: false },
    { name: "Layout/TrailingWhitespace", lines: 500, violations: true },
    { name: "Layout/TrailingWhitespace", lines: 1000, violations: false },
    { name: "Layout/TrailingWhitespace", lines: 1000, violations: true },
  ]

  puts "## Layout/TrailingWhitespace Benchmark"
  puts
  puts "| Lines | Violations | RuboCop (ms) | Reukocyte (ms) | Speedup |"
  puts "|-------|------------|--------------|----------------|---------|"

  tests.each do |test|
    file_path = File.join(temp_dir, "test_#{test[:lines]}_#{test[:violations]}.rb")
    File.write(file_path, generate_ruby_file(test[:lines], with_violations: test[:violations]))

    # Warmup RuboCop
    WARMUP.times { run_rubocop(file_path, cops: ['Layout/TrailingWhitespace']) }

    # Benchmark RuboCop
    rubocop_times = []
    ITERATIONS.times do
      start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      run_rubocop(file_path, cops: ['Layout/TrailingWhitespace'])
      elapsed = Process.clock_gettime(Process::CLOCK_MONOTONIC) - start
      rubocop_times << elapsed * 1000 # Convert to ms
    end
    rubocop_avg = rubocop_times.sum / rubocop_times.size

    # For Reukocyte, we'll measure using the Rust benchmark
    # Since we don't have a CLI yet, we'll estimate from cargo bench results
    # For now, show RuboCop baseline only

    violations_str = test[:violations] ? "Yes" : "No"
    puts "| #{test[:lines]} | #{violations_str} | #{rubocop_avg.round(2)} | (see cargo bench) | - |"
  end

  puts
  puts "## Lint/Debugger Benchmark"
  puts
  puts "| Methods | Debuggers | RuboCop (ms) |"
  puts "|---------|-----------|--------------|"

  lint_tests = [
    { methods: 50, debuggers: false },
    { methods: 50, debuggers: true },
    { methods: 100, debuggers: false },
    { methods: 100, debuggers: true },
  ]

  lint_tests.each do |test|
    file_path = File.join(temp_dir, "lint_#{test[:methods]}_#{test[:debuggers]}.rb")
    File.write(file_path, generate_debugger_file(test[:methods], with_debuggers: test[:debuggers]))

    # Warmup
    WARMUP.times { run_rubocop(file_path, cops: ['Lint/Debugger']) }

    # Benchmark RuboCop
    rubocop_times = []
    ITERATIONS.times do
      start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      run_rubocop(file_path, cops: ['Lint/Debugger'])
      elapsed = Process.clock_gettime(Process::CLOCK_MONOTONIC) - start
      rubocop_times << elapsed * 1000
    end
    rubocop_avg = rubocop_times.sum / rubocop_times.size

    debuggers_str = test[:debuggers] ? "Yes" : "No"
    puts "| #{test[:methods]} | #{debuggers_str} | #{rubocop_avg.round(2)} |"
  end

ensure
  FileUtils.rm_rf(temp_dir)
end

puts
puts "Note: Reukocyte times can be obtained from 'cargo bench'"
puts "RuboCop version: #{`rubocop --version`.strip}"
