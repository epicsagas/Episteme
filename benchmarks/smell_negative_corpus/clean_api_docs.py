# Guards against: Comments Smell FP.
# A module with thorough docstrings is well-documented, not over-commented.
# Public API modules should document every function — this is a feature.

from __future__ import annotations

from typing import Sequence


def calculate_tax(income: float, rate: float, deductions: Sequence[float]) -> float:
    """Calculate income tax with deductions applied.

    Computes taxable income by subtracting all deductions from gross income,
    then applies the flat tax rate. Taxable income is clamped to zero —
    negative results are not carried forward.

    Args:
        income: Annual income in USD (must be non-negative).
        rate: Tax rate as a decimal between 0.0 and 1.0.
        deductions: Sequence of deduction amounts in USD.

    Returns:
        Tax amount in USD, rounded to two decimal places.

    Raises:
        ValueError: If income is negative or rate is outside [0.0, 1.0].
    """
    if income < 0:
        raise ValueError("income must be non-negative")
    if not 0.0 <= rate <= 1.0:
        raise ValueError("rate must be between 0.0 and 1.0")

    total_deductions = sum(deductions)
    taxable = max(0.0, income - total_deductions)
    return round(taxable * rate, 2)


def calculate_compound_interest(
    principal: float,
    annual_rate: float,
    years: int,
    compounds_per_year: int = 12,
) -> float:
    """Calculate compound interest over a fixed period.

    Uses the standard compound interest formula:
        A = P * (1 + r/n)^(n*t)

    Args:
        principal: Initial investment amount in USD.
        annual_rate: Annual interest rate as decimal (e.g., 0.05 for 5%).
        years: Number of years to compound.
        compounds_per_year: Compounding frequency (default 12 = monthly).

    Returns:
        Final amount after compounding, rounded to two decimal places.
    """
    return round(
        principal * (1 + annual_rate / compounds_per_year)
        ** (compounds_per_year * years),
        2,
    )


def calculate_mortgage_payment(
    principal: float,
    annual_rate: float,
    years: int,
) -> float:
    """Calculate fixed monthly mortgage payment.

    Args:
        principal: Loan amount in USD.
        annual_rate: Annual interest rate as decimal.
        years: Loan term in years.

    Returns:
        Monthly payment in USD, rounded to two decimal places.
    """
    if annual_rate == 0:
        return round(principal / (years * 12), 2)

    monthly_rate = annual_rate / 12
    num_payments = years * 12
    payment = principal * (
        monthly_rate * (1 + monthly_rate) ** num_payments
    ) / ((1 + monthly_rate) ** num_payments - 1)
    return round(payment, 2)
