#!/usr/bin/env ruby
# Benchmark comparing PreCop vs RuboCop (CLI) vs RuboCop (LSP/Server mode)
#
# RuboCop Server mode (--server) keeps a daemon running to avoid startup cost.
# This is similar to how LSP mode works.

require 'benchmark'
require 'fileutils'
require 'socket'
require 'timeout'

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

def run_precop(files)
  files_arg = Array(files).map { |f| %("#{f}") }.join(' ')
  `#{PRECOP} #{files_arg} -q 2>&1`
end

def run_rubocop_cli(files)
  files_arg = Array(files).map { |f| %("#{f}") }.join(' ')
  `rubocop -c "#{RUBOCOP_CONFIG}" #{files_arg} --format quiet 2>&1`
end

def run_rubocop_server(files)
  files_arg = Array(files).map { |f| %("#{f}") }.join(' ')
  `rubocop -c "#{RUBOCOP_CONFIG}" --server #{files_arg} --format quiet 2>&1`
end

def server_running?
  # Check if RuboCop server is running
  `rubocop --server-status 2>&1`.include?('running')
end

def start_server
  puts "Starting RuboCop server..."
  # Start server in background
  spawn("rubocop --start-server -c #{RUBOCOP_CONFIG}", [:out, :err] => '/dev/null')

  # Wait for server to be ready
  30.times do
    if server_running?
      puts "RuboCop server started."
      return true
    end
    sleep 0.1
  end

  puts "Warning: Could not confirm server started"
  false
end

def stop_server
  `rubocop --stop-server 2>&1`
  sleep 0.2
end

puts "=" * 70
puts "PreCop vs RuboCop Benchmark (CLI vs Server Mode)"
puts "=" * 70
puts ""

# Ensure server is stopped initially
stop_server

# Check if server mode is available
server_available = system('rubocop --help 2>&1 | grep -q "\-\-server"')
unless server_available
  puts "Note: RuboCop server mode requires RuboCop 1.31+"
  puts "      Checking version..."
end

rubocop_version = `rubocop --version`.strip
puts "RuboCop version: #{rubocop_version}"
puts ""

# Start RuboCop server
start_server

# Warm up
puts "Warming up..."
small_file = File.join(TEST_FILES_DIR, 'trailing_whitespace_small.rb')
run_precop(small_file)
run_rubocop_cli(small_file)
run_rubocop_server(small_file)  # First call warms up the server
run_rubocop_server(small_file)  # Second call uses warm server
puts ""

results = []
iterations = 5

# Test different file sizes
%w[small medium large].each do |size|
  file = File.join(TEST_FILES_DIR, "trailing_whitespace_#{size}.rb")
  next unless File.exist?(file)

  file_size = File.size(file)

  precop_times = []
  rubocop_cli_times = []
  rubocop_server_times = []

  iterations.times do
    precop_times << Benchmark.realtime { run_precop(file) }
    rubocop_cli_times << Benchmark.realtime { run_rubocop_cli(file) }
    rubocop_server_times << Benchmark.realtime { run_rubocop_server(file) }
  end

  precop_avg = (precop_times.sum / iterations * 1000).round(2)
  rubocop_cli_avg = (rubocop_cli_times.sum / iterations * 1000).round(2)
  rubocop_server_avg = (rubocop_server_times.sum / iterations * 1000).round(2)

  results << {
    name: "trailing_ws_#{size}.rb",
    size: file_size,
    precop: precop_avg,
    rubocop_cli: rubocop_cli_avg,
    rubocop_server: rubocop_server_avg
  }
end

# Test many files
many_files_dir = File.join(TEST_FILES_DIR, 'many_files')
if Dir.exist?(many_files_dir)
  files = Dir.glob(File.join(many_files_dir, '*.rb'))

  precop_times = []
  rubocop_cli_times = []
  rubocop_server_times = []

  iterations.times do
    precop_times << Benchmark.realtime { run_precop(files) }
    rubocop_cli_times << Benchmark.realtime { run_rubocop_cli(files) }
    rubocop_server_times << Benchmark.realtime { run_rubocop_server(files) }
  end

  precop_avg = (precop_times.sum / iterations * 1000).round(2)
  rubocop_cli_avg = (rubocop_cli_times.sum / iterations * 1000).round(2)
  rubocop_server_avg = (rubocop_server_times.sum / iterations * 1000).round(2)

  results << {
    name: "100 files",
    size: files.sum { |f| File.size(f) },
    precop: precop_avg,
    rubocop_cli: rubocop_cli_avg,
    rubocop_server: rubocop_server_avg
  }
end

# Print results
puts "Results (average of #{iterations} runs):"
puts "-" * 85
printf "%-20s %8s %12s %14s %14s\n",
       "Test", "Size", "PreCop", "RuboCop CLI", "RuboCop Server"
puts "-" * 85

results.each do |r|
  size_str = r[:size] < 1024 ? "#{r[:size]} B" : "#{(r[:size] / 1024.0).round(1)} KB"
  printf "%-20s %8s %10.2f ms %12.2f ms %12.2f ms\n",
         r[:name], size_str, r[:precop], r[:rubocop_cli], r[:rubocop_server]
end

puts "-" * 85
puts ""

# Speedup comparison
puts "Speedup Analysis:"
puts "-" * 85
printf "%-20s %20s %20s %20s\n",
       "Test", "CLI vs PreCop", "Server vs PreCop", "CLI vs Server"
puts "-" * 85

results.each do |r|
  cli_vs_precop = (r[:rubocop_cli] / r[:precop]).round(1)
  server_vs_precop = (r[:rubocop_server] / r[:precop]).round(1)
  cli_vs_server = (r[:rubocop_cli] / r[:rubocop_server]).round(1)

  printf "%-20s %18.1fx slower %18.1fx slower %18.1fx faster\n",
         r[:name], cli_vs_precop, server_vs_precop, cli_vs_server
end

puts "-" * 85
puts ""

# Summary
avg_cli_speedup = (results.sum { |r| r[:rubocop_cli] / r[:precop] } / results.size).round(1)
avg_server_speedup = (results.sum { |r| r[:rubocop_server] / r[:precop] } / results.size).round(1)
avg_server_improvement = (results.sum { |r| r[:rubocop_cli] / r[:rubocop_server] } / results.size).round(1)

puts "Summary:"
puts "  - PreCop vs RuboCop CLI:    #{avg_cli_speedup}x faster"
puts "  - PreCop vs RuboCop Server: #{avg_server_speedup}x faster"
puts "  - Server vs CLI:            #{avg_server_improvement}x faster (RuboCop server mode improvement)"
puts ""

# Stop server
stop_server
puts "RuboCop server stopped."

# Cleanup
File.delete(RUBOCOP_CONFIG) if File.exist?(RUBOCOP_CONFIG)
