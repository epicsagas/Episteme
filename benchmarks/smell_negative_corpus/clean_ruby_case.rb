# Guards against: Switch Statements FP.
# Ruby case/when with type matching is the idiomatic way to dispatch on
# error types — each branch is a single line.

# frozen_string_literal: true

# Custom error types for domain-specific handling.
class ValidationError < StandardError
  attr_reader :field

  def initialize(field, message)
    @field = field
    super(message)
  end
end

class AuthError < StandardError; end
class NotFoundError < StandardError; end
class RateLimitError < StandardError
  attr_reader :retry_after

  def initialize(retry_after)
    @retry_after = retry_after
    super("rate limited, retry after #{retry_after}s")
  end
end
class TimeoutError < StandardError; end
class PermissionError < StandardError; end

# Format an error into a user-friendly message.
# Each branch is a single expression — the case is proportional to the
# number of error types and is not a switch smell.
def format_error(error)
  case error
  when ValidationError then "Validation: #{error.message} (field: #{error.field})"
  when AuthError then "Authentication: #{error.message}"
  when NotFoundError then "Not found: #{error.message}"
  when RateLimitError then "Rate limited: retry after #{error.retry_after}s"
  when TimeoutError then "Timeout: #{error.message}"
  when PermissionError then "Permission denied: #{error.message}"
  else "Error: #{error.message}"
  end
end

# Return an HTTP status code for an error type.
def error_status(error)
  case error
  when ValidationError then 422
  when AuthError then 401
  when NotFoundError then 404
  when RateLimitError then 429
  when TimeoutError then 504
  when PermissionError then 403
  else 500
  end
end
