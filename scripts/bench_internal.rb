#!/usr/bin/env ruby
# frozen_string_literal: true

require 'benchmark'
require 'rubocop'

def generate_file(lines)
  content = lines.times.map { |i| "x = #{i}   " }.join("\n")
  path = "/tmp/test_#{lines}.rb"
  File.write(path, content)
  path
end

puts "RuboCop Internal API Benchmark (no startup overhead)"
puts "=" * 50

[100, 500, 1000, 5000].each do |lines|
  path = generate_file(lines)
  source = File.read(path)
  config = RuboCop::ConfigStore.new.for(path)

  # Warmup
  5.times do
    ps = RuboCop::ProcessedSource.new(source, 3.3, path)
    cop = RuboCop::Cop::Layout::TrailingWhitespace.new(config)
    comm = RuboCop::Cop::Commissioner.new([cop], [], raise_error: true)
    comm.investigate(ps)
  end

  # Measure
  times = []
  30.times do
    time = Benchmark.realtime do
      ps = RuboCop::ProcessedSource.new(source, 3.3, path)
      cop = RuboCop::Cop::Layout::TrailingWhitespace.new(config)
      comm = RuboCop::Cop::Commissioner.new([cop], [], raise_error: true)
      comm.investigate(ps)
    end
    times << time * 1000
  end

  # Remove outliers
  times.sort!
  trimmed = times[5..-6]
  avg = trimmed.sum / trimmed.size

  puts "Layout/TrailingWhitespace #{lines} lines: #{'%.3f' % avg} ms"
end

puts
puts "Lint/Debugger Benchmark"
puts "=" * 50

def generate_lint_file(methods)
  content = ["class Test"]
  methods.times do |i|
    content << "  def m#{i}"
    content << "    binding.pry" if i % 5 == 0
    content << "    x = 1"
    content << "  end"
  end
  content << "end"
  path = "/tmp/lint_#{methods}.rb"
  File.write(path, content.join("\n"))
  path
end

[50, 100, 500].each do |methods|
  path = generate_lint_file(methods)
  source = File.read(path)
  config = RuboCop::ConfigStore.new.for(path)

  # Warmup
  5.times do
    ps = RuboCop::ProcessedSource.new(source, 3.3, path)
    cop = RuboCop::Cop::Lint::Debugger.new(config)
    comm = RuboCop::Cop::Commissioner.new([cop], [], raise_error: true)
    comm.investigate(ps)
  end

  # Measure
  times = []
  30.times do
    time = Benchmark.realtime do
      ps = RuboCop::ProcessedSource.new(source, 3.3, path)
      cop = RuboCop::Cop::Lint::Debugger.new(config)
      comm = RuboCop::Cop::Commissioner.new([cop], [], raise_error: true)
      comm.investigate(ps)
    end
    times << time * 1000
  end

  times.sort!
  trimmed = times[5..-6]
  avg = trimmed.sum / trimmed.size

  puts "Lint/Debugger #{methods} methods: #{'%.3f' % avg} ms"
end
