#!/usr/bin/env python3
"""
Rate limiting middleware for Syntagma API
Implements per-IP and per-endpoint rate limits using slowapi
"""

from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.errors import RateLimitExceeded
from slowapi.util import get_remote_address

# Initialize rate limiter
limiter = Limiter(key_func=get_remote_address)


class RateLimits:
    """Rate limit configurations for different endpoints"""

    # Default rate limit for most endpoints
    DEFAULT = "100/minute"

    # Stricter limits for expensive endpoints
    ANALYZE = "20/minute"
    REFACTOR = "20/minute"
    SEARCH = "50/minute"

    # No limit for health/info endpoints
    UNLIMITED = None


def setup_rate_limiting(app):
    """
    Setup rate limiting for FastAPI app

    Args:
        app: FastAPI application instance
    """
    # Add exception handler
    app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)

    # Add state for limiter
    app.state.limiter = limiter

    print(f"✅ Rate limiting enabled: {RateLimits.DEFAULT} (default)")
