# Guards against: Data Class FP.
# A frozen dataclass with rich type annotations is the canonical way to
# represent structured data in Python — it's not a smell, it's the point.

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional


@dataclass(frozen=True)
class Address:
    """Immutable postal address value object.

    Fields are intentionally bare — this is a data container that maps to
    external address validation APIs and database rows.
    """

    street: str
    city: str
    state: str
    zip_code: str
    country: str = "US"
    apartment: Optional[str] = None
    is_primary: bool = True


@dataclass(frozen=True)
class PhoneNumber:
    """Phone number with type classification."""

    number: str
    kind: str = "mobile"  # mobile, home, work, fax
    country_code: str = "+1"
    is_verified: bool = False


@dataclass(frozen=True)
class ContactInfo:
    """Aggregated contact information for a person."""

    email: str
    phone: PhoneNumber
    address: Address
    preferred_contact: str = "email"  # email, phone, mail


def format_address(addr: Address) -> str:
    """Format an address into a single-line string."""
    parts = [addr.street]
    if addr.apartment:
        parts.append(f"Apt {addr.apartment}")
    parts.append(f"{addr.city}, {addr.state} {addr.zip_code}")
    parts.append(addr.country)
    return ", ".join(parts)


def format_phone(phone: PhoneNumber) -> str:
    """Format a phone number for display."""
    return f"{phone.country_code} {phone.number} ({phone.kind})"
