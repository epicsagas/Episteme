#!/usr/bin/env python3
"""Test Redis caching functionality"""

import asyncio
import time
import uuid

import pytest

from syntagma.api.cache import cache_manager, cached


# Use a unique prefix per test session to avoid stale key collisions
_SESSION_PREFIX = f"test_func_{uuid.uuid4().hex[:8]}"

call_count = 0


@cached(prefix=_SESSION_PREFIX, ttl=60)
async def expensive_computation(x: int, y: int) -> int:
    """Simulate expensive computation — tracks call count to detect cache misses."""
    global call_count
    call_count += 1
    await asyncio.sleep(0.1)  # Simulate work
    return x + y


def test_cache_miss_then_hit():
    """First call (miss) executes the function body; second call (hit) returns cached value
    without executing the body. Verified via call_count side-effect and >= 2x timing speedup."""
    global call_count

    async def _run():
        try:
            await cache_manager.connect()
        except Exception:
            return "skip:Redis not available"

        stats = await cache_manager.get_stats()
        if not stats.get("connected", False):
            await cache_manager.disconnect()
            return "skip:Redis not available"

        try:
            # Use fresh args to guarantee a real cache miss on first call
            arg_x, arg_y = int(uuid.uuid4().int % 1000), int(uuid.uuid4().int % 1000)

            count_before = call_count

            # First call — cache miss: expensive_computation body must run
            start = time.time()
            result1 = await expensive_computation(arg_x, arg_y)
            duration1 = time.time() - start

            count_after_first = call_count

            # Second call — cache hit: body must NOT run again
            start = time.time()
            result2 = await expensive_computation(arg_x, arg_y)
            duration2 = time.time() - start

            count_after_second = call_count

            return (result1, result2, duration1, duration2,
                    count_before, count_after_first, count_after_second,
                    arg_x + arg_y)
        finally:
            await cache_manager.disconnect()

    outcome = asyncio.run(_run())

    if isinstance(outcome, str) and outcome.startswith("skip:"):
        pytest.skip(outcome[len("skip:"):])

    (result1, result2, duration1, duration2,
     count_before, count_after_first, count_after_second, expected) = outcome

    # Both calls must return the correct value
    assert result1 == expected
    assert result2 == expected

    # The function body must have run exactly once (on the miss)
    assert count_after_first == count_before + 1, (
        "Expected exactly 1 new call on cache miss"
    )
    assert count_after_second == count_after_first, (
        "Expected 0 new calls on cache hit (body should not execute)"
    )

    # Timing sanity: miss includes asyncio.sleep(0.1), hit should be much faster
    if duration2 > 0:
        speedup = duration1 / duration2
        assert speedup >= 2, (
            f"Expected >= 2x speedup, got {speedup:.1f}x "
            f"(miss={duration1*1000:.1f}ms, hit={duration2*1000:.1f}ms)"
        )


def test_cache_stats():
    """Stats dict must contain the 'enabled' key."""
    async def _run():
        try:
            await cache_manager.connect()
        except Exception:
            return "skip:Redis not available"

        try:
            stats = await cache_manager.get_stats()
            return stats
        finally:
            await cache_manager.disconnect()

    outcome = asyncio.run(_run())

    if isinstance(outcome, str) and outcome.startswith("skip:"):
        pytest.skip(outcome[len("skip:"):])

    stats = outcome
    assert isinstance(stats, dict), "get_stats() must return a dict"
    assert "enabled" in stats, f"'enabled' key missing from stats: {stats}"
