#!/usr/bin/env python3
"""
Structured JSON Logging Configuration for Syntagma API
Provides request ID tracking, structured logging, and production-ready log formatting
"""

import json
import logging
import sys
import time
import uuid
from contextvars import ContextVar
from datetime import datetime, timezone
from typing import Any, Dict, Optional

from fastapi import Request
from starlette.middleware.base import BaseHTTPMiddleware

# ===== Context Variables for Request Tracking =====

request_id_ctx: ContextVar[Optional[str]] = ContextVar("request_id", default=None)
request_context_ctx: ContextVar[Optional[Dict[str, Any]]] = ContextVar(
    "request_context", default=None
)


# ===== JSON Formatter =====


class JSONFormatter(logging.Formatter):
    """
    Custom formatter that outputs logs as JSON
    """

    def format(self, record: logging.LogRecord) -> str:
        """
        Format log record as JSON

        Args:
            record: Log record to format

        Returns:
            JSON string
        """
        log_data = {
            "timestamp": datetime.fromtimestamp(record.created, tz=timezone.utc).isoformat(),
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
        }

        # Add request ID if available
        request_id = request_id_ctx.get()
        if request_id:
            log_data["request_id"] = request_id

        # Add request context if available
        request_context = request_context_ctx.get()
        if request_context:
            log_data.update(request_context)

        # Add exception info if present
        if record.exc_info:
            log_data["exception"] = self.formatException(record.exc_info)

        # Add extra fields
        if hasattr(record, "extra"):
            log_data.update(record.extra)

        return json.dumps(log_data, ensure_ascii=False)


# ===== Logger Setup =====


def setup_logging(
    level: str = "INFO", enable_json: bool = True, logger_name: str = "syntagma"
) -> logging.Logger:
    """
    Configure structured logging

    Args:
        level: Logging level (DEBUG, INFO, WARNING, ERROR)
        enable_json: Whether to use JSON formatting (False for dev)
        logger_name: Root logger name

    Returns:
        Configured logger instance
    """
    # Get or create logger
    logger = logging.getLogger(logger_name)
    logger.setLevel(getattr(logging, level.upper()))

    # Remove existing handlers
    logger.handlers.clear()

    # Create console handler
    handler = logging.StreamHandler(sys.stdout)

    # Set formatter
    fmt: logging.Formatter
    if enable_json:
        fmt = JSONFormatter()
    else:
        # Simple format for development
        fmt = logging.Formatter(
            "%(asctime)s - %(name)s - %(levelname)s - %(message)s", datefmt="%Y-%m-%d %H:%M:%S"
        )
    formatter = fmt

    handler.setFormatter(formatter)
    logger.addHandler(handler)

    # Don't propagate to root logger
    logger.propagate = False

    return logger


def get_logger(name: str = "syntagma") -> logging.Logger:
    """
    Get a logger instance

    Args:
        name: Logger name (will be nested under root logger)

    Returns:
        Logger instance
    """
    return logging.getLogger(name)


# ===== Request ID Middleware =====


class RequestIDMiddleware(BaseHTTPMiddleware):
    """
    Middleware that adds request_id to all requests and logs
    """

    async def dispatch(self, request: Request, call_next):
        """
        Process request with request ID tracking

        Args:
            request: Incoming request
            call_next: Next middleware/handler

        Returns:
            Response with request ID header
        """
        # Generate or extract request ID
        request_id = request.headers.get("X-Request-ID", str(uuid.uuid4()))

        # Set context variable
        request_id_ctx.set(request_id)

        # Build request context
        request_context = {
            "method": request.method,
            "path": request.url.path,
        }

        # Extract API key ID if present (for rate limiting tracking)
        api_key = request.headers.get("X-API-Key")
        if api_key:
            # Only store first 8 chars for privacy
            request_context["api_key_id"] = api_key[:8] + "..."

        request_context_ctx.set(request_context)

        # Process request
        start_time = time.time()
        response = await call_next(request)
        duration_ms = (time.time() - start_time) * 1000

        # Add request ID to response headers
        response.headers["X-Request-ID"] = request_id

        # Log request completion
        logger = get_logger("syntagma.api")

        log_extra = {
            "request_id": request_id,
            "method": request.method,
            "path": request.url.path,
            "status_code": response.status_code,
            "duration_ms": round(duration_ms, 2),
        }

        # Determine log level based on status code
        if response.status_code >= 500:
            logger.error("Request completed", extra=log_extra)
        elif response.status_code >= 400:
            logger.warning("Request completed", extra=log_extra)
        else:
            logger.info("Request completed", extra=log_extra)

        return response


# ===== Helper Functions =====


def log_with_context(logger: logging.Logger, level: str, message: str, **extra_fields):
    """
    Log with additional context fields

    Args:
        logger: Logger instance
        level: Log level (info, warning, error, debug)
        message: Log message
        **extra_fields: Additional fields to include in log
    """
    log_func = getattr(logger, level.lower())
    log_func(message, extra=extra_fields)


def log_analysis(
    logger: logging.Logger, smells_detected: int, duration_ms: float, code_length: int
):
    """
    Log code analysis operation

    Args:
        logger: Logger instance
        smells_detected: Number of smells detected
        duration_ms: Analysis duration in milliseconds
        code_length: Length of analyzed code
    """
    log_with_context(
        logger,
        "info",
        "Code analysis completed",
        smells_detected=smells_detected,
        duration_ms=round(duration_ms, 2),
        code_length=code_length,
    )


def log_search(
    logger: logging.Logger,
    query: str,
    results_count: int,
    duration_ms: float,
    entity_type: Optional[str] = None,
):
    """
    Log semantic search operation

    Args:
        logger: Logger instance
        query: Search query
        results_count: Number of results returned
        duration_ms: Search duration in milliseconds
        entity_type: Entity type filter (if any)
    """
    log_with_context(
        logger,
        "info",
        "Semantic search completed",
        query_length=len(query),
        results_count=results_count,
        duration_ms=round(duration_ms, 2),
        entity_type=entity_type or "all",
    )


def log_error(logger: logging.Logger, error: Exception, context: str, **extra_fields):
    """
    Log error with context

    Args:
        logger: Logger instance
        error: Exception that occurred
        context: Description of what was being done
        **extra_fields: Additional context fields
    """
    log_with_context(
        logger,
        "error",
        f"Error during {context}: {str(error)}",
        error_type=type(error).__name__,
        **extra_fields,
    )
