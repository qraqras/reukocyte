#!/usr/bin/env ruby
# Benchmark comparing PreCop vs RuboCop (CLI) vs RuboCop (Server mode)
#
# RuboCop Server mode keeps a daemon running to eliminate startup cost.

require 'benchmark'
require 'fileutils'

SCRIPT_DIR = File.dirname(__FILE__)
PROJECT_ROOT = File.dirname(SCRIPT_DIR)
TEST_FILES_DIR = File.join(SCRIPT_DIR, 'test_files')
PRECOP = File.join(PROJECT_ROOT, 'target', 'release', 'precop')

# Create a temporary directory with .rubocop.yml for server mode
TEMP_DIR = File.join(SCRIPT_DIR, 'temp_bench')
FileUtils.mkdir_p(TEMP_DIR)

# Copy test files to temp dir
FileUtils.cp_r(TEST_FILES_DIR, File.join(TEMP_DIR, 'test_files'))
TEST_DIR = File.join(TEMP_DIR, 'test_files')

# RuboCop config - placed in temp dir so server picks it up
RUBOCOP_CONFIG = File.join(TEMP_DIR, '.rubocop.yml')
File.write(RUBOCOP_CONFIG, <<~YAML)
  AllCops:
    DisabledByDefault: true
    SuggestExtensions: false
    NewCops: disable

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
  `cd #{TEMP_DIR} && rubocop #{files_arg} --format quiet 2>&1`
end

def run_rubocop_server(files)
  files_arg = Array(files).map { |f| %("#{f}") }.join(' ')
  `cd #{TEMP_DIR} && rubocop #{files_arg} --format quiet 2>&1`
end

def stop_server
  Dir.chdir(TEMP_DIR) do
    `rubocop --stop-server 2>&1`
  end
  sleep 0.3
end

def start_server
  puts "Starting RuboCop server..."
  Dir.chdir(TEMP_DIR) do
    # Start server in background
    pid = spawn("rubocop --start-server", [:out, :err] => '/dev/null')
    Process.detach(pid)
  end

  # Wait for server to be ready
  20.times do
    result = `cd #{TEMP_DIR} && rubocop --server-status 2>&1`
    if result.include?('running')
      puts "RuboCop server started."
      return true
    end
    sleep 0.2
  end

  puts "Warning: Could not confirm server started"
  false
end

puts "=" * 75
puts "PreCop vs RuboCop Benchmark (CLI vs Server Mode)"
puts "=" * 75
puts ""

# Ensure server is stopped initially
stop_server

rubocop_version = `rubocop --version`.strip
puts "RuboCop version: #{rubocop_version}"
puts "Working directory: #{TEMP_DIR}"
puts ""

# === Phase 1: CLI Mode (no server) ===
puts "Phase 1: RuboCop CLI mode (no server running)"
puts "-" * 75

cli_results = []
iterations = 5

%w[small medium large].each do |size|
  file = File.join(TEST_DIR, "trailing_whitespace_#{size}.rb")
  next unless File.exist?(file)

  file_size = File.size(file)

  precop_times = []
  rubocop_times = []

  iterations.times do
    precop_times << Benchmark.realtime { run_precop(file) }
    rubocop_times << Benchmark.realtime { run_rubocop_cli(file) }
  end

  cli_results << {
    name: "trailing_ws_#{size}.rb",
    size: file_size,
    precop: (precop_times.sum / iterations * 1000).round(2),
    rubocop: (rubocop_times.sum / iterations * 1000).round(2)
  }
end

# Many files test
many_files = Dir.glob(File.join(TEST_DIR, 'many_files', '*.rb'))
if many_files.any?
  precop_times = []
  rubocop_times = []

  iterations.times do
    precop_times << Benchmark.realtime { run_precop(many_files) }
    rubocop_times << Benchmark.realtime { run_rubocop_cli(many_files) }
  end

  cli_results << {
    name: "100 files",
    size: many_files.sum { |f| File.size(f) },
    precop: (precop_times.sum / iterations * 1000).round(2),
    rubocop: (rubocop_times.sum / iterations * 1000).round(2)
  }
end

# === Phase 2: Server Mode ===
puts ""
puts "Phase 2: RuboCop Server mode"
puts "-" * 75

start_server

# Warm up the server
puts "Warming up server..."
3.times { run_rubocop_server(File.join(TEST_DIR, "trailing_whitespace_small.rb")) }
puts ""

server_results = []

%w[small medium large].each do |size|
  file = File.join(TEST_DIR, "trailing_whitespace_#{size}.rb")
  next unless File.exist?(file)

  file_size = File.size(file)

  precop_times = []
  rubocop_times = []

  iterations.times do
    precop_times << Benchmark.realtime { run_precop(file) }
    rubocop_times << Benchmark.realtime { run_rubocop_server(file) }
  end

  server_results << {
    name: "trailing_ws_#{size}.rb",
    size: file_size,
    precop: (precop_times.sum / iterations * 1000).round(2),
    rubocop: (rubocop_times.sum / iterations * 1000).round(2)
  }
end

# Many files test with server
if many_files.any?
  precop_times = []
  rubocop_times = []

  iterations.times do
    precop_times << Benchmark.realtime { run_precop(many_files) }
    rubocop_times << Benchmark.realtime { run_rubocop_server(many_files) }
  end

  server_results << {
    name: "100 files",
    size: many_files.sum { |f| File.size(f) },
    precop: (precop_times.sum / iterations * 1000).round(2),
    rubocop: (rubocop_times.sum / iterations * 1000).round(2)
  }
end

stop_server

# === Results ===
puts ""
puts "=" * 75
puts "RESULTS (average of #{iterations} runs)"
puts "=" * 75
puts ""

puts "CLI Mode (RuboCop without server):"
puts "-" * 75
printf "%-22s %8s %12s %14s %12s\n", "Test", "Size", "PreCop", "RuboCop CLI", "Speedup"
puts "-" * 75

cli_results.each do |r|
  size_str = r[:size] < 1024 ? "#{r[:size]} B" : "#{(r[:size] / 1024.0).round(1)} KB"
  speedup = (r[:rubocop] / r[:precop]).round(1)
  printf "%-22s %8s %10.2f ms %12.2f ms %11.1fx\n",
         r[:name], size_str, r[:precop], r[:rubocop], speedup
end

puts ""
puts "Server Mode (RuboCop with server running):"
puts "-" * 75
printf "%-22s %8s %12s %14s %12s\n", "Test", "Size", "PreCop", "RuboCop Srv", "Speedup"
puts "-" * 75

server_results.each do |r|
  size_str = r[:size] < 1024 ? "#{r[:size]} B" : "#{(r[:size] / 1024.0).round(1)} KB"
  speedup = (r[:rubocop] / r[:precop]).round(1)
  printf "%-22s %8s %10.2f ms %12.2f ms %11.1fx\n",
         r[:name], size_str, r[:precop], r[:rubocop], speedup
end

puts ""
puts "=" * 75
puts "COMPARISON SUMMARY"
puts "=" * 75
puts ""

printf "%-22s %14s %14s %14s\n", "Test", "RuboCop CLI", "RuboCop Srv", "Server Speedup"
puts "-" * 75

cli_results.zip(server_results).each do |cli, srv|
  improvement = (cli[:rubocop] / srv[:rubocop]).round(1)
  printf "%-22s %12.2f ms %12.2f ms %13.1fx\n",
         cli[:name], cli[:rubocop], srv[:rubocop], improvement
end

puts ""
avg_cli_speedup = (cli_results.sum { |r| r[:rubocop] / r[:precop] } / cli_results.size).round(1)
avg_server_speedup = (server_results.sum { |r| r[:rubocop] / r[:precop] } / server_results.size).round(1)
avg_server_improvement = (cli_results.zip(server_results).sum { |c, s| c[:rubocop] / s[:rubocop] } / cli_results.size).round(1)

puts "Averages:"
puts "  PreCop vs RuboCop CLI:     #{avg_cli_speedup}x faster"
puts "  PreCop vs RuboCop Server:  #{avg_server_speedup}x faster"
puts "  Server mode improvement:   #{avg_server_improvement}x (vs CLI)"
puts ""

# Cleanup
FileUtils.rm_rf(TEMP_DIR)
puts "Cleanup complete."
