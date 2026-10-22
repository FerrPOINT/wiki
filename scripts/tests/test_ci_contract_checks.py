#!/usr/bin/env python3
"""Regression checks for Wiki API contract gates in CI."""

from __future__ import annotations

import unittest
import json
import runpy
import shlex
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/release.yml"


class CiContractChecksTest(unittest.TestCase):
    def test_backend_job_runs_postgres_tests_against_isolated_service(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        backend_job = workflow.split("\n  backend:\n", 1)[1].split("\n  frontend:\n", 1)[0]
        self.assertIn("image: postgres:16-alpine", backend_job)
        self.assertIn("POSTGRES_HOST_AUTH_METHOD: trust", backend_job)
        self.assertIn("cargo test --locked --workspace -- --test-threads=1", backend_job)
        self.assertIn(
            "WIKI_TEST_DATABASE_URL: postgres://wiki@127.0.0.1:5432/wiki_test",
            backend_job,
        )

    def test_frontend_job_checks_openapi_backward_compatibility(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        frontend_job = workflow.split("\n  frontend:\n", 1)[1].split("\n  deny:\n", 1)[0]
        self.assertIn("fetch-depth: 0", frontend_job)
        self.assertIn("pnpm install --frozen-lockfile --ignore-scripts=false", frontend_job)
        self.assertIn("git diff --exit-code -- pnpm-lock.yaml src/api/generated.ts src/api/generated-exports.ts", frontend_job)
        self.assertIn("pnpm openapi:compat", frontend_job)
        self.assertIn("pnpm format:check", frontend_job)


class BackendUnitGraphTests(unittest.TestCase):
    def test_graph_checks_have_a_scoped_entrypoint_and_existing_ci_owner(self):
        selector = runpy.run_path(str(ROOT / "scripts/vendor/base_quality_selection.py"))
        plan = selector["plan"](ROOT, profile="build", changed=["scripts/tests/test_ci_contract_checks.py"])
        self.assertEqual([gate["id"] for gate in plan["gates"]], ["tooling:build-contracts"])
        manifest = json.loads((ROOT / ".quality-gate.json").read_text())
        gate = next(g for g in manifest["gates"] if g["id"] == "tooling:build-contracts")
        self.assertIn("backend/Dockerfile*", gate["inputs"])
        self.assertEqual(gate["test_tier"], "static")
        command = "python3 -m unittest scripts.tests.test_verify_readme scripts.tests.test_ci_contract_checks"
        self.assertEqual((ROOT / ".github/workflows/ci.yml").read_text().count(command), 1)

    @staticmethod
    def stages(text):
        """Follow FROM, COPY and RUN-bind edges of these literal recipes."""
        result = {}
        current = None
        for line in re.sub(r"\\\r?\n", " ", text).splitlines():
            if line.startswith("FROM "):
                words = shlex.split(line)
                current = words[-1] if len(words) == 4 and words[2].upper() == "AS" else "runtime"
                if current in result:
                    raise ValueError("Duplicate Docker stage")
                result[current] = {"parent": words[1], "body": []}
            elif current is not None:
                result[current]["body"].append(line)
        return result

    @classmethod
    def closure(cls, stages, target):
        seen = set()
        def visit(name):
            if name in seen:
                return
            seen.add(name)
            stage = stages[name]
            dependencies = {stage["parent"]}
            for line in stage["body"]:
                if line.startswith(("COPY ", "RUN ")):
                    dependencies.update(re.findall(r"(?:--from=|\bfrom=)([a-zA-Z0-9_-]+)", line))
            for dependency in dependencies & stages.keys():
                visit(dependency)
        visit(target)
        return seen

    def test_unit_target_never_reaches_server_dependency_compilation(self):
        for name in ("Dockerfile", "Dockerfile.umbrella"):
            with self.subTest(recipe=name):
                graph = self.stages((ROOT / "backend" / name).read_text(encoding="utf-8"))
                reachable = self.closure(graph, "unit")
                commands = "\n".join(line for stage in reachable for line in graph[stage]["body"])
                self.assertNotIn("dependencies", reachable)
                self.assertNotIn("cargo build", commands)
                self.assertIn("cargo test --locked --release -p domain --lib --no-run", commands)
                self.assertIn("cargo test --locked --release -p domain --lib &&", commands)
                self.assertIn("sharing=locked", commands)

    def test_runtime_keeps_server_dependencies_and_the_required_unit_barrier(self):
        for name in ("Dockerfile", "Dockerfile.umbrella"):
            with self.subTest(recipe=name):
                graph = self.stages((ROOT / "backend" / name).read_text(encoding="utf-8"))
                self.assertTrue({"unit", "dependencies", "builder"} <= self.closure(graph, "runtime"))
                dependencies = "\n".join(graph["dependencies"]["body"])
                self.assertIn("cargo build --locked --release --bin server", dependencies)
                self.assertIn("cargo clean --release", dependencies)
                builder = "\n".join(graph["builder"]["body"])
                self.assertIn("from=unit,source=/tmp/rust-unit-check", builder)
                self.assertIn('test "$(cat /tmp/rust-unit-check)" = passed', builder)
                self.assertIn("cargo build --locked --release --bin server", builder)


if __name__ == "__main__":
    unittest.main()
