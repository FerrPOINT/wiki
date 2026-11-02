"""Cargo worker arguments must reach the builder, not only the CLI."""
from pathlib import Path
import unittest
ROOT=Path(__file__).resolve().parents[2]
class DockerfileBudgetTests(unittest.TestCase):
    def test_each_rust_recipe_consumes_worker_budget_before_cargo(self):
        for name in ("Dockerfile", "Dockerfile.umbrella"):
            with self.subTest(recipe=name):
                builder, runtime=(ROOT/"backend"/name).read_text().split("FROM debian:bookworm-slim",1)
                self.assertIn("ARG CARGO_BUILD_JOBS=1",builder.splitlines())
                self.assertLess(builder.index("ARG CARGO_BUILD_JOBS=1"),builder.index("cargo build"))
                self.assertNotIn("CARGO_BUILD_JOBS",runtime)
if __name__=="__main__":unittest.main()
