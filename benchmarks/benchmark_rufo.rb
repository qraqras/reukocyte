#!/usr/bin/env ruby
# Benchmark: PreCop vs Rufo vs RuboCop (Formatters comparison)
#
# Comparing Ruby formatting/layout tools:
# - PreCop: Rust-based Layout linter (check mode)
# - Rufo: Ruby formatter
# - RuboCop: Ruby linter with Layout cops

require 'benchmark'
require 'fileutils'

SCRIPT_DIR = File.dirname(__FILE__)
PROJECT_ROOT = File.dirname(SCRIPT_DIR)
TEST_FILES_DIR = File.join(SCRIPT_DIR, 'test_files')
PRECOP = File.join(PROJECT_ROOT, 'target', 'release', 'precop')

# Create proper Ruby files for Rufo (it needs valid Ruby syntax)
TEMP_DIR = File.join(SCRIPT_DIR, 'temp_rufo_bench')
FileUtils.rm_rf(TEMP_DIR)
FileUtils.mkdir_p(TEMP_DIR)

def generate_ruby_file(path, num_methods)
  content = <<~RUBY
    # frozen_string_literal: true

    class BenchmarkClass
  RUBY

  num_methods.times do |i|
    # Add some formatting issues
    if i % 3 == 0
      content += <<~RUBY
          def method_#{i}(arg1,arg2,arg3)
             puts "method #{i}"
            result = arg1+arg2+arg3
            return result
          end

      RUBY
    else
      content += <<~RUBY
          def method_#{i}
            puts "method #{i}"
          end

      RUBY
    end
  end

  content += "end\n"
  File.write(path, content)
end

# Generate test files
puts "Generating test files..."
test_files = {
  small: 20,
  medium: 100,
  large: 500
}

test_files.each do |size, methods|
  generate_ruby_file(File.join(TEMP_DIR, "test_#{size}.rb"), methods)
end

# Generate many small files
many_dir = File.join(TEMP_DIR, 'many_files')
FileUtils.mkdir_p(many_dir)
100.times do |i|
  generate_ruby_file(File.join(many_dir, "file_#{i}.rb"), 10)
end

puts "Test files generated."
puts ""

def run_precop(files)
  files_arg = Array(files).map { |f| %("#{f}") }.join(' ')
  `#{PRECOP} #{files_arg} -q 2>&1`
end

def run_rufo_check(files)
  files_arg = Array(files).map { |f| %("#{f}") }.join(' ')
  `rufo --check #{files_arg} 2>&1`
end

def run_rubocop_layout(files)
  files_arg = Array(files).map { |f| %("#{f}") }.join(' ')
  config = <<~YAML
    AllCops:
      DisabledByDefault: true
      SuggestExtensions: false
      NewCops: disable
    Layout:
      Enabled: true
  YAML
  config_file = File.join(TEMP_DIR, '.rubocop.yml')
  File.write(config_file, config)
  `rubocop -c "#{config_file}" #{files_arg} --format quiet 2>&1`
end

puts "=" * 80
puts "Benchmark: PreCop vs Rufo vs RuboCop (Layout Check Mode)"
puts "=" * 80
puts ""
puts "Tools:"
puts "  - PreCop #{`#{PRECOP} --version 2>&1`.strip rescue 'dev'}"
puts "  - Rufo #{`rufo --version`.strip}"
puts "  - RuboCop #{`rubocop --version`.strip}"
puts ""

# Warm up
puts "Warming up..."
small_file = File.join(TEMP_DIR, 'test_small.rb')
run_precop(small_file)
run_rufo_check(small_file)
run_rubocop_layout(small_file)
puts ""

results = []
iterations = 5

test_cases = [
  { name: "small (20 methods)", file: File.join(TEMP_DIR, 'test_small.rb') },
  { name: "medium (100 methods)", file: File.join(TEMP_DIR, 'test_medium.rb') },
  { name: "large (500 methods)", file: File.join(TEMP_DIR, 'test_large.rb') },
]

# Many files
many_files = Dir.glob(File.join(many_dir, '*.rb'))
test_cases << { name: "100 small files", file: many_files }

test_cases.each do |test|
  puts "Testing: #{test[:name]}..."

  files = test[:file]
  file_size = Array(files).sum { |f| File.size(f) }

  precop_times = []
  rufo_times = []
  rubocop_times = []

  iterations.times do
    precop_times << Benchmark.realtime { run_precop(files) }
    rufo_times << Benchmark.realtime { run_rufo_check(files) }
    rubocop_times << Benchmark.realtime { run_rubocop_layout(files) }
  end

  results << {
    name: test[:name],
    size: file_size,
    precop: (precop_times.sum / iterations * 1000).round(2),
    rufo: (rufo_times.sum / iterations * 1000).round(2),
    rubocop: (rubocop_times.sum / iterations * 1000).round(2)
  }
end

puts ""
puts "=" * 80
puts "RESULTS (average of #{iterations} runs)"
puts "=" * 80
puts ""

puts "-" * 80
printf "%-22s %10s %12s %12s %14s\n", "Test", "Size", "PreCop", "Rufo", "RuboCop"
puts "-" * 80

results.each do |r|
  size_str = r[:size] < 1024 ? "#{r[:size]} B" : "#{(r[:size] / 1024.0).round(1)} KB"
  printf "%-22s %10s %10.1f ms %10.1f ms %12.1f ms\n",
         r[:name], size_str, r[:precop], r[:rufo], r[:rubocop]
end

puts ""
puts "=" * 80
puts "SPEEDUP COMPARISON"
puts "=" * 80
puts ""

puts "-" * 80
printf "%-22s %18s %18s %18s\n", "Test", "PreCop vs Rufo", "PreCop vs RuboCop", "Rufo vs RuboCop"
puts "-" * 80

results.each do |r|
  precop_vs_rufo = (r[:rufo] / r[:precop]).round(1)
  precop_vs_rubocop = (r[:rubocop] / r[:precop]).round(1)
  rufo_vs_rubocop = (r[:rubocop] / r[:rufo]).round(1)

  printf "%-22s %15.1fx faster %15.1fx faster %15.1fx faster\n",
         r[:name], precop_vs_rufo, precop_vs_rubocop, rufo_vs_rubocop
end

puts ""
puts "=" * 80
puts "SUMMARY"
puts "=" * 80
puts ""

avg_vs_rufo = (results.sum { |r| r[:rufo] / r[:precop] } / results.size).round(1)
avg_vs_rubocop = (results.sum { |r| r[:rubocop] / r[:precop] } / results.size).round(1)
avg_rufo_vs_rubocop = (results.sum { |r| r[:rubocop] / r[:rufo] } / results.size).round(1)

puts "Average speedup:"
puts "  PreCop vs Rufo:    #{avg_vs_rufo}x faster"
puts "  PreCop vs RuboCop: #{avg_vs_rubocop}x faster"
puts "  Rufo vs RuboCop:   #{avg_rufo_vs_rubocop}x faster"
puts ""

puts "Characteristics:"
puts "  PreCop:  Rust-based, check only (fix WIP), Layout cops"
puts "  Rufo:    Ruby-based, format + check, opinionated formatter"
puts "  RuboCop: Ruby-based, check + fix, configurable Layout cops"
puts ""

# Test fix/format mode times (if available)
puts "=" * 80
puts "FORMAT/FIX MODE COMPARISON"
puts "=" * 80
puts ""

# Copy files for format test
format_dir = File.join(TEMP_DIR, 'format_test')
FileUtils.mkdir_p(format_dir)
FileUtils.cp(File.join(TEMP_DIR, 'test_medium.rb'), File.join(format_dir, 'test.rb'))

puts "Testing format/fix mode on medium file (100 methods)..."

# Rufo format
rufo_format_times = []
iterations.times do
  FileUtils.cp(File.join(TEMP_DIR, 'test_medium.rb'), File.join(format_dir, 'test.rb'))
  rufo_format_times << Benchmark.realtime { `rufo "#{File.join(format_dir, 'test.rb')}" 2>&1` }
end
rufo_format_avg = (rufo_format_times.sum / iterations * 1000).round(2)

# RuboCop autocorrect
rubocop_fix_times = []
config_file = File.join(TEMP_DIR, '.rubocop.yml')
iterations.times do
  FileUtils.cp(File.join(TEMP_DIR, 'test_medium.rb'), File.join(format_dir, 'test.rb'))
  rubocop_fix_times << Benchmark.realtime {
    `rubocop -c "#{config_file}" --autocorrect "#{File.join(format_dir, 'test.rb')}" --format quiet 2>&1`
  }
end
rubocop_fix_avg = (rubocop_fix_times.sum / iterations * 1000).round(2)

puts ""
puts "-" * 50
printf "%-20s %15s\n", "Tool", "Format Time"
puts "-" * 50
printf "%-20s %13.1f ms\n", "Rufo (format)", rufo_format_avg
printf "%-20s %13.1f ms\n", "RuboCop (-a)", rubocop_fix_avg
printf "%-20s %13s\n", "PreCop (--fix)", "(not yet implemented)"
puts "-" * 50
puts ""

puts "Rufo is #{(rubocop_fix_avg / rufo_format_avg).round(1)}x faster than RuboCop for formatting."
puts ""

# Cleanup
FileUtils.rm_rf(TEMP_DIR)
puts "Cleanup complete."
