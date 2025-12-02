# frozen_string_literal: true

# This file contains debugger statements for testing

class Example
  def method_with_binding_pry
    value = 42
    binding.pry
    process(value)
  end

  def method_with_byebug
    value = 42
    byebug
    process(value)
  end

  def method_with_debugger
    value = 42
    debugger
    process(value)
  end

  def clean_method
    value = 42
    process(value)
  end

  def method_with_binding_irb
    binding.irb
    do_something
  end
end

Pry.rescue do
  risky_operation
end
