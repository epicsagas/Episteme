"""Hatchling build hook — injects telemetry keys into the wheel at build time.

At ``python -m build`` time this hook reads two environment variables
and writes ``src/syntagma/_keys.py`` into the wheel so that released
binaries have the keys baked in without any runtime env-var requirement.

If either variable is absent (local dev, sdist build) the file is still
written but with empty strings, so the import never fails.

GitHub Actions must set::

    env:
      POSTHOG_KEY: ${{ secrets.POSTHOG_KEY }}
      SENTRY_DSN_SYNTAGMA: ${{ secrets.SENTRY_DSN_SYNTAGMA }}

on the ``python -m build`` step.
"""

from __future__ import annotations

import os
from pathlib import Path

from hatchling.builders.hooks.plugin.interface import (  # pyright: ignore[reportMissingImports]
    BuildHookInterface,
)


class CustomBuildHook(BuildHookInterface):
    def initialize(self, version: str, build_data: dict) -> None:  # noqa: ARG002
        posthog_key = os.environ.get("POSTHOG_KEY", "")
        sentry_dsn = os.environ.get("SENTRY_DSN_SYNTAGMA", "")

        keys_path = Path(self.root) / "src" / "syntagma" / "_keys.py"
        keys_path.write_text(
            f'POSTHOG_KEY = "{posthog_key}"\n'
            f'SENTRY_DSN_SYNTAGMA = "{sentry_dsn}"\n',
            encoding="utf-8",
        )
