# frozen_string_literal: true

# This file contains trailing whitespace violations for testing

class Example
  def method_with_trailing_spaces   
    value = 42   
    process(value)
  end

  def clean_method
    value = 42
    process(value)
  end

  def another_method_with_spaces   
    result = compute   
  end
end
