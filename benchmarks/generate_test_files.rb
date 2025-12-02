#!/usr/bin/env ruby
# Generate test Ruby files for benchmarking

require 'fileutils'

OUTPUT_DIR = File.join(__dir__, 'test_files')
FileUtils.mkdir_p(OUTPUT_DIR)

# Generate files with trailing whitespace issues
def generate_trailing_whitespace_file(num_lines)
  lines = []
  num_lines.times do |i|
    if i % 3 == 0
      lines << "  puts 'line #{i}'   "  # trailing whitespace
    else
      lines << "  puts 'line #{i}'"
    end
  end

  <<~RUBY
class TrailingWhitespaceExample
  def example_method
#{lines.join("\n")}
  end
end
  RUBY
end

# Generate files with indentation issues
def generate_indentation_file(num_methods)
  methods = []
  num_methods.times do |i|
    # Some methods with correct indentation, some with wrong
    if i % 4 == 0
      methods << <<~RUBY
        def method_#{i}
           puts 'wrong indent'
        end
      RUBY
    else
      methods << <<~RUBY
        def method_#{i}
          puts 'correct indent'
        end
      RUBY
    end
  end

  <<~RUBY
class IndentationExample
#{methods.join("\n")}
end
  RUBY
end

# Generate multiple files
SIZES = {
  small: 100,
  medium: 1000,
  large: 5000
}

SIZES.each do |size_name, count|
  # Trailing whitespace files
  content = generate_trailing_whitespace_file(count)
  File.write(File.join(OUTPUT_DIR, "trailing_whitespace_#{size_name}.rb"), content)

  # Indentation files
  content = generate_indentation_file(count / 5)
  File.write(File.join(OUTPUT_DIR, "indentation_#{size_name}.rb"), content)
end

# Generate many small files for parallel processing test
many_files_dir = File.join(OUTPUT_DIR, 'many_files')
FileUtils.mkdir_p(many_files_dir)

100.times do |i|
  content = generate_trailing_whitespace_file(50)
  File.write(File.join(many_files_dir, "file_#{i}.rb"), content)
end

puts "Generated test files in #{OUTPUT_DIR}"
puts "Files:"
Dir.glob(File.join(OUTPUT_DIR, '*.rb')).each { |f| puts "  #{File.basename(f)} (#{File.size(f)} bytes)" }
puts "Many files: #{Dir.glob(File.join(many_files_dir, '*.rb')).count} files"
