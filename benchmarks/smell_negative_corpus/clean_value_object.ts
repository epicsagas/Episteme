// Guards against: Data Class FP.
// Value objects and DTOs with readonly fields are the correct way to model
// immutable data in TypeScript — they should not contain business logic.

/**
 * Represents a monetary amount in a specific currency.
 *
 * This is a pure value object — it carries data and enforces immutability
 * via `readonly`. No methods are needed because operations on Money
 * belong in a MoneyService or similar domain service.
 */
interface Money {
  readonly amount: number;
  readonly currency: string;
}

/**
 * Geographic coordinates as an immutable value.
 */
interface Coordinates {
  readonly latitude: number;
  readonly longitude: number;
}

/**
 * Date range with inclusive start and exclusive end.
 */
interface DateRange {
  readonly start: string; // ISO 8601
  readonly end: string;
}

/**
 * Pagination parameters for list queries.
 */
interface Pagination {
  readonly page: number;
  readonly perPage: number;
}

/**
 * API error envelope returned on failures.
 */
interface ApiError {
  readonly code: string;
  readonly message: string;
  readonly details?: ReadonlyArray<readonly { readonly field: string; readonly reason: string }>;
}

/**
 * Factory functions for creating value objects with validation.
 */
function createMoney(amount: number, currency: string): Money {
  if (amount < 0) {
    throw new Error("Amount cannot be negative");
  }
  if (currency.length !== 3) {
    throw new Error("Currency must be a 3-letter ISO code");
  }
  return Object.freeze({ amount, currency });
}

function createPagination(page: number, perPage: number): Pagination {
  if (page < 1) throw new Error("Page must be >= 1");
  if (perPage < 1 || perPage > 100) throw new Error("PerPage must be 1-100");
  return Object.freeze({ page, perPage });
}

export { Money, Coordinates, DateRange, Pagination, ApiError, createMoney, createPagination };
