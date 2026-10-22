#!/usr/bin/env python3
"""Regression checks for Wiki API contract gates in CI."""

from __future__ import annotations

import unittest
import json
import runpy
import re
import shlex
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


class BackendUnitBoundaryTests(unittest.TestCase):
    TEST_FILES = ("backend/domain/src/value_objects/tests.rs", "backend/domain/src/wiki/tests.rs")

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

    def test_both_recipes_keep_tests_in_unit_and_out_of_production(self):
        for name in ("Dockerfile", "Dockerfile.umbrella"):
            text = (ROOT / "backend" / name).read_text(encoding="utf-8")
            unit = text.split("FROM unit-dependencies AS unit\n", 1)[1].split("FROM dependencies AS builder\n", 1)[0]
            builder = text.split("FROM dependencies AS builder\n", 1)[1].split("FROM debian:", 1)[0]
            self.assertIn("COPY wiki/backend/domain ./domain", unit)
            lines = [shlex.split(line) for line in builder.splitlines() if line.startswith("COPY ") and "wiki/backend/domain" in line]
            self.assertEqual(len(lines), 1)
            excludes = {word.removeprefix("--exclude=") for word in lines[0] if word.startswith("--exclude=")}
            self.assertEqual(excludes, {"**/src/value_objects/tests.rs", "**/src/wiki/tests.rs"})

    def test_all_thirteen_existing_units_stay_behind_cfg_test_modules(self):
        count = 0
        for name in self.TEST_FILES:
            source = ROOT / name
            count += source.read_text(encoding="utf-8").count("#[test]")
            owner = source.parent.with_suffix(".rs")
            include = source.relative_to(owner.parent).as_posix()
            self.assertIn('#[cfg(test)]\n#[path = "' + include + '"]\nmod tests;', owner.read_text(encoding="utf-8"))
        self.assertEqual(count, 13)

    def test_installed_selector_keeps_each_test_edit_out_of_artifacts(self):
        selector = runpy.run_path(str(ROOT / "scripts/vendor/base_quality_selection.py"))
        for path in self.TEST_FILES:
            plan = selector["plan"](ROOT, profile="build", changed=[path])
            self.assertNotIn("backend:build", {g["id"] for g in plan["gates"]})
            units = selector["plan"](ROOT, profile="unit", changed=[path])
            self.assertTrue(any(g.get("test_tier") == "unit" and g["component"] == "backend" for g in units["gates"]))
        for path in ("backend/domain/src/wiki.rs", "backend/domain/src/value_objects.rs"):
            plan = selector["plan"](ROOT, profile="build", changed=[path])
            self.assertIn("backend:build", {g["id"] for g in plan["gates"]})

    def test_artifact_exclusions_are_exact_known_test_files(self):
        manifest = json.loads((ROOT / ".quality-gate.json").read_text(encoding="utf-8"))
        artifact = next(g for g in manifest["gates"] if g["id"] == "backend:build")
        for path in self.TEST_FILES:
            self.assertIn(path, artifact["exclude_inputs"])
            self.assertIn(path, manifest["backend_unit_inputs"])

    def test_build_contract_module_selects_only_its_pure_tooling_gate(self):
        selector = runpy.run_path(str(ROOT / "scripts/vendor/base_quality_selection.py"))
        plan = selector["plan"](ROOT, profile="build", changed=["scripts/tests/test_ci_contract_checks.py"])
        self.assertEqual([g["id"] for g in plan["gates"]], ["tooling:build-contracts"])


if __name__ == "__main__":
    unittest.main()
