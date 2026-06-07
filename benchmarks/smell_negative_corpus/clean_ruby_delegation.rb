# Guards against: Middle Man FP.
# Ruby's SimpleDelegator is the standard pattern for creating decorator-like
# wrappers — delegation to the wrapped object is the entire purpose.

# frozen_string_literal: true

require "delegate"

# LoggedArray wraps an Array and logs mutating operations.
# The delegation via SimpleDelegator is intentional — it exists to add
# logging cross-cutting behavior, not to mindlessly forward calls.
class LoggedArray < SimpleDelegator
  def initialize(array = [])
    super
  end

  def push(*items)
    $stderr.puts "pushing #{items.size} items"
    super
  end

  def pop
    $stderr.puts "popping one item"
    super
  end

  def <<(item)
    $stderr.puts "appending #{item.inspect}"
    super
  end
end

# CountedHash wraps a Hash and tracks how many times values are looked up.
class CountedHash < SimpleDelegator
  def initialize(hash = {})
    super
    @access_count = 0
  end

  def [](key)
    @access_count += 1
    super
  end

  def fetch(key, *args)
    @access_count += 1
    super
  end

  attr_reader :access_count
end
