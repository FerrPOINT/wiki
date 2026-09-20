#!/usr/bin/env python3
"""Regression checks for Wiki API contract gates in CI."""

from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/ci.yml"


class CiContractChecksTest(unittest.TestCase):
    def test_backend_job_runs_postgres_tests_against_isolated_service(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        backend_job = workflow.split("\n  backend:\n", 1)[1].split("\n  frontend:\n", 1)[0]
        self.assertIn("image: postgres:16-alpine", backend_job)
        self.assertIn("POSTGRES_HOST_AUTH_METHOD: trust", backend_job)
        self.assertIn("cargo test --workspace -- --test-threads=1", backend_job)
        self.assertIn(
            "WIKI_TEST_DATABASE_URL: postgres://wiki@127.0.0.1:5432/wiki_test",
            backend_job,
        )

    def test_frontend_job_checks_openapi_backward_compatibility(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        frontend_job = workflow.split("\n  frontend:\n", 1)[1].split("\n  deny:\n", 1)[0]
        self.assertIn("fetch-depth: 0", frontend_job)
        self.assertIn("pnpm openapi:check", frontend_job)
        self.assertIn("pnpm openapi:compat", frontend_job)
        self.assertIn("pnpm format:check", frontend_job)


if __name__ == "__main__":
    unittest.main()
