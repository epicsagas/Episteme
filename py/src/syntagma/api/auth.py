#!/usr/bin/env python3
"""
Authentication middleware for Syntagma API
Implements API key authentication with environment-based key storage
"""

from typing import Optional

from fastapi import HTTPException, Security, status
from fastapi.security import APIKeyHeader
from starlette.requests import Request

from syntagma.config import API_KEYS

# API Key header scheme
api_key_header = APIKeyHeader(name="X-API-Key", auto_error=False)


class APIKeyAuth:
    """API Key authentication handler"""

    def __init__(self):
        """Initialize with keys from environment variable"""
        self.enabled = self._load_api_keys()

    def _load_api_keys(self) -> bool:
        """
        Load API keys from SYNTAGMA_API_KEYS environment variable

        Returns:
            bool: True if keys loaded, False if running in dev mode (no keys)
        """
        keys_str = API_KEYS

        if not keys_str:
            print("⚠️  No SYNTAGMA_API_KEYS found - running in DEV mode (no auth)")
            self.api_keys = set()
            return False

        # Parse comma-separated keys
        self.api_keys = {key.strip() for key in keys_str.split(",") if key.strip()}

        if not self.api_keys:
            print("⚠️  SYNTAGMA_API_KEYS is empty - running in DEV mode (no auth)")
            return False

        print(f"🔐 Loaded {len(self.api_keys)} API key(s) from environment")
        return True

    def verify_api_key(self, api_key: Optional[str] = Security(api_key_header)) -> Optional[str]:
        """
        Verify API key from request header

        Args:
            api_key: API key from X-API-Key header

        Returns:
            str: API key identifier (first 8 chars) if valid, None if dev mode

        Raises:
            HTTPException: 401 if auth enabled and key missing/invalid
        """
        # Dev mode: no authentication required
        if not self.enabled:
            return None

        # Auth enabled: key is required
        if not api_key:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Missing API key. Provide X-API-Key header.",
                headers={"WWW-Authenticate": "ApiKey"},
            )

        # Validate key
        if api_key not in self.api_keys:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Invalid API key",
                headers={"WWW-Authenticate": "ApiKey"},
            )

        # Return key identifier for logging (first 8 chars)
        return api_key[:8]

    def get_api_key_info(self, request: Request) -> Optional[str]:
        """
        Extract API key identifier from request for logging

        Args:
            request: FastAPI request object

        Returns:
            str: API key identifier or "dev-mode" or "missing"
        """
        if not self.enabled:
            return "dev-mode"

        api_key = request.headers.get("X-API-Key")
        if not api_key:
            return "missing"

        if api_key in self.api_keys:
            return api_key[:8]

        return "invalid"


# Global auth instance
auth_handler = APIKeyAuth()
