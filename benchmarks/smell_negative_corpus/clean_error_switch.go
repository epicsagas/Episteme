// Guards against: Switch Statements FP.
// Go type switch on error types is idiomatic error handling — each case is
// a single line, and the compiler does not provide exhaustiveness for errors.

package negative

import "fmt"

// ValidationError indicates invalid input data.
type ValidationError struct {
	Field string
	Msg   string
}

func (e *ValidationError) Error() string {
	return fmt.Sprintf("validation failed on %s: %s", e.Field, e.Msg)
}

// AuthError indicates an authentication failure.
type AuthError struct {
	Reason string
}

func (e *AuthError) Error() string {
	return fmt.Sprintf("auth failed: %s", e.Reason)
}

// NotFoundError indicates a missing resource.
type NotFoundError struct {
	Resource string
	ID       string
}

func (e *NotFoundError) Error() string {
	return fmt.Sprintf("%s %s not found", e.Resource, e.ID)
}

// RateLimitError indicates the client has exceeded rate limits.
type RateLimitError struct {
	RetryAfterSec int
}

func (e *RateLimitError) Error() string {
	return fmt.Sprintf("rate limited, retry after %ds", e.RetryAfterSec)
}

// TimeoutError indicates an operation exceeded its deadline.
type TimeoutError struct {
	Operation string
}

func (e *TimeoutError) Error() string {
	return fmt.Sprintf("operation %s timed out", e.Operation)
}

// handleError returns a user-facing error message based on error type.
// Each case maps to a single response — this is not a "switch smell"
// because Go error handling is inherently type-discriminated.
func handleError(err error) string {
	switch err.(type) {
	case *ValidationError:
		return "validation failed"
	case *AuthError:
		return "authentication failed"
	case *NotFoundError:
		return "resource not found"
	case *RateLimitError:
		return "rate limited"
	case *TimeoutError:
		return "request timed out"
	default:
		return "unknown error"
	}
}
