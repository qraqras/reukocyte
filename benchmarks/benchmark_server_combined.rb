#!/usr/bin/env ruby
# Benchmark: PreCop + RuboCop Server vs RuboCop Server Only
#
# This tests the scenario where RuboCop Server is already running,
# eliminating startup cost. This is the best-case scenario for combination.

require 'benchmark'
require 'fileutils'

SCRIPT_DIR = File.dirname(__FILE__)
PROJECT_ROOT = File.dirname(SCRIPT_DIR)
TEST_FILES_DIR = File.join(SCRIPT_DIR, 'test_files')
PRECOP = File.join(PROJECT_ROOT, 'target', 'release', 'precop')

TEMP_DIR = File.join(SCRIPT_DIR, 'temp_server_bench')
FileUtils.rm_rf(TEMP_DIR)
FileUtils.mkdir_p(TEMP_DIR)
FileUtils.cp_r(TEST_FILES_DIR, File.join(TEMP_DIR, 'test_files'))
TEST_DIR = File.join(TEMP_DIR, 'test_files')

def run_precop(files)
  files_arg = Array(files).map { |f| %("#{f}") }.join(' ')
  `#{PRECOP} #{files_arg} -q 2>&1`
end

def stop_all_servers
  `rubocop --stop-server 2>&1`
  sleep 0.3
end

def create_config_dir(name, config_content)
  dir = File.join(TEMP_DIR, name)
  FileUtils.mkdir_p(dir)
  File.write(File.join(dir, '.rubocop.yml'), config_content)
  # Symlink test files
  FileUtils.ln_sf(TEST_DIR, File.join(dir, 'test_files'))
  dir
end

# Create separate directories for each config (server uses .rubocop.yml in cwd)
all_cops_dir = create_config_dir('all_cops', <<~YAML)
  AllCops:
    SuggestExtensions: false
    NewCops: disable
YAML

no_layout_dir = create_config_dir('no_layout', <<~YAML)
  AllCops:
    SuggestExtensions: false
    NewCops: disable
  Layout:
    Enabled: false
YAML

layout_only_dir = create_config_dir('layout_only', <<~YAML)
  AllCops:
    DisabledByDefault: true
    SuggestExtensions: false
    NewCops: disable
  Layout:
    Enabled: true
YAML

puts "=" * 80
puts "Benchmark: PreCop + RuboCop Server vs RuboCop Server Only"
puts "=" * 80
puts ""
puts "This measures the benefit when RuboCop Server eliminates startup cost."
puts ""

stop_all_servers

# Test function that uses server
def run_rubocop_server(dir, files)
  files_arg = Array(files).map { |f| %("#{f}") }.join(' ')
  `cd "#{dir}" && rubocop #{files_arg} --format quiet 2>&1`
end

def start_server(dir)
  pid = spawn("cd #{dir} && rubocop --start-server", [:out, :err] => '/dev/null')
  Process.detach(pid)
  20.times do
    result = `cd #{dir} && rubocop --server-status 2>&1`
    return true if result.include?('running')
    sleep 0.2
  end
  false
end

# Start servers for each config
puts "Starting RuboCop servers..."
start_server(all_cops_dir)
start_server(no_layout_dir)
start_server(layout_only_dir)
puts "Servers started."
puts ""

# Warm up
puts "Warming up..."
small_file = File.join(TEST_DIR, 'trailing_whitespace_small.rb')
3.times do
  run_precop(small_file)
  run_rubocop_server(all_cops_dir, "test_files/trailing_whitespace_small.rb")
  run_rubocop_server(no_layout_dir, "test_files/trailing_whitespace_small.rb")
  run_rubocop_server(layout_only_dir, "test_files/trailing_whitespace_small.rb")
end
puts ""

results = []
iterations = 5

test_cases = [
  { name: "small (1.8KB)", file: "trailing_whitespace_small.rb" },
  { name: "medium (18.5KB)", file: "trailing_whitespace_medium.rb" },
  { name: "large (96.6KB)", file: "trailing_whitespace_large.rb" },
]

# Add many files test
many_files = Dir.glob(File.join(TEST_DIR, 'many_files', '*.rb')).map { |f| "test_files/many_files/#{File.basename(f)}" }
test_cases << { name: "100 files", file: many_files } if many_files.any?

test_cases.each do |test|
  puts "Testing: #{test[:name]}..."

  files = test[:file]
  full_path = Array(files).map { |f| File.join(TEST_DIR, f.sub('test_files/', '')) }

  precop_times = []
  server_all_times = []
  server_layout_times = []
  server_no_layout_times = []

  iterations.times do
    precop_times << Benchmark.realtime { run_precop(full_path) }
    server_all_times << Benchmark.realtime { run_rubocop_server(all_cops_dir, files) }
    server_layout_times << Benchmark.realtime { run_rubocop_server(layout_only_dir, files) }
    server_no_layout_times << Benchmark.realtime { run_rubocop_server(no_layout_dir, files) }
  end

  precop_avg = (precop_times.sum / iterations * 1000).round(2)
  server_all_avg = (server_all_times.sum / iterations * 1000).round(2)
  server_layout_avg = (server_layout_times.sum / iterations * 1000).round(2)
  server_no_layout_avg = (server_no_layout_times.sum / iterations * 1000).round(2)
  combined_avg = precop_avg + server_no_layout_avg

  results << {
    name: test[:name],
    precop: precop_avg,
    server_all: server_all_avg,
    server_layout: server_layout_avg,
    server_no_layout: server_no_layout_avg,
    combined: combined_avg
  }
end

# Stop servers
puts ""
puts "Stopping servers..."
Dir.chdir(all_cops_dir) { `rubocop --stop-server 2>&1` }
Dir.chdir(no_layout_dir) { `rubocop --stop-server 2>&1` }
Dir.chdir(layout_only_dir) { `rubocop --stop-server 2>&1` }

puts ""
puts "=" * 80
puts "RESULTS (average of #{iterations} runs) - Server Mode"
puts "=" * 80
puts ""

puts "Execution Times (with RuboCop Server - no startup cost):"
puts "-" * 80
printf "%-15s %10s %12s %12s %12s %12s\n",
       "Test", "PreCop", "Srv All", "Srv Layout", "Srv NoLay", "PreCop+Srv"
puts "-" * 80

results.each do |r|
  printf "%-15s %8.1f ms %10.1f ms %10.1f ms %10.1f ms %10.1f ms\n",
         r[:name], r[:precop], r[:server_all], r[:server_layout], r[:server_no_layout], r[:combined]
end

puts ""
puts "=" * 80
puts "COMPARISON: PreCop+RuboCop Server vs RuboCop Server Only"
puts "=" * 80
puts ""

puts "-" * 80
printf "%-15s %12s %12s %12s %10s\n",
       "Test", "Srv All", "PreCop+Srv", "Saved", "Speedup"
puts "-" * 80

total_all = 0
total_combined = 0

results.each do |r|
  saved = r[:server_all] - r[:combined]
  speedup = r[:server_all] / r[:combined]
  total_all += r[:server_all]
  total_combined += r[:combined]

  printf "%-15s %10.1f ms %10.1f ms %10.1f ms %9.2fx\n",
         r[:name], r[:server_all], r[:combined], saved, speedup
end

puts "-" * 80
total_saved = total_all - total_combined
total_speedup = total_all / total_combined
printf "%-15s %10.1f ms %10.1f ms %10.1f ms %9.2fx\n",
       "TOTAL", total_all, total_combined, total_saved, total_speedup

puts ""
puts "=" * 80
puts "LAYOUT COP COMPARISON (Server Mode)"
puts "=" * 80
puts ""

puts "-" * 80
printf "%-15s %14s %10s %10s\n", "Test", "Srv Layout", "PreCop", "Speedup"
puts "-" * 80

results.each do |r|
  speedup = r[:server_layout] / r[:precop]
  printf "%-15s %12.1f ms %8.1f ms %9.1fx\n",
         r[:name], r[:server_layout], r[:precop], speedup
end

puts ""
puts "=" * 80
puts "SUMMARY"
puts "=" * 80
puts ""

avg_layout_speedup = (results.sum { |r| r[:server_layout] / r[:precop] } / results.size).round(1)
pct_saved = ((total_saved / total_all) * 100).round(1)

puts "With RuboCop Server (startup cost eliminated):"
puts ""
puts "  Layout cops:"
puts "    RuboCop Server: #{results.map { |r| r[:server_layout] }.sum.round(0)} ms"
puts "    PreCop:         #{results.map { |r| r[:precop] }.sum.round(0)} ms"
puts "    Speedup:        #{avg_layout_speedup}x"
puts ""
puts "  Full pipeline (all cops):"
puts "    RuboCop Server only:     #{total_all.round(0)} ms"
puts "    PreCop + RuboCop Server: #{total_combined.round(0)} ms"
puts "    Time saved:              #{total_saved.round(0)} ms (#{pct_saved}%)"
puts "    Speedup:                 #{total_speedup.round(2)}x"
puts ""

# Cleanup
FileUtils.rm_rf(TEMP_DIR)
puts "Cleanup complete."
