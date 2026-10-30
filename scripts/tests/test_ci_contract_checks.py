#!/usr/bin/env python3
"""Regression checks for Wiki API contract gates in CI."""

from __future__ import annotations

import unittest
import hashlib
import json
import runpy
import re
import shlex
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/release.yml"


class CiContractChecksTest(unittest.TestCase):
    def test_swagger_assets_are_locked_cargo_inputs_without_compiler_network(self):
        manifest = tomllib.loads((ROOT / "backend/Cargo.toml").read_text(encoding="utf-8"))
        dependency = manifest["workspace"]["dependencies"]["utoipa-swagger-ui"]
        self.assertIn("axum", dependency["features"])
        self.assertIn("vendored", dependency["features"])
        packages = tomllib.loads((ROOT / "backend/Cargo.lock").read_text(encoding="utf-8"))["package"]
        swagger = next(item for item in packages if item["name"] == "utoipa-swagger-ui")
        self.assertIn("utoipa-swagger-ui-vendored", swagger["dependencies"])
        vendored = next(item for item in packages if item["name"] == "utoipa-swagger-ui-vendored")
        self.assertEqual(vendored["version"], "0.1.2")  # Keeps the original Swagger UI 5.17.14.
        self.assertRegex(vendored["checksum"], r"^[a-f0-9]{64}$")
        routes = (ROOT / "backend/api/src/lib.rs").read_text(encoding="utf-8")
        self.assertIn('SwaggerUi::new("/swagger-ui")', routes)

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
        self.assertTrue("git ls-files --error-unmatch pnpm-lock.yaml" in frontend_job)
        self.assertTrue("git diff --exit-code -- pnpm-lock.yaml" in frontend_job)
        for generated in ("src/api/generated.ts", "src/api/generated-exports.ts"):
            for check in (f"git check-ignore -q -- {generated}", f"test -f {generated}",
                          f"test ! -L {generated}", f"test -s {generated}"):
                self.assertTrue(check in frontend_job, f"Missing generated-output guard: {check}")
        self.assertTrue("pnpm openapi:check" in frontend_job, "Ignored outputs still need independent content parity")
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


class FrontendGenerationContractTests(unittest.TestCase):
    def test_docker_stage_consumes_the_registered_helper_before_postinstall(self):
        for name in ('Dockerfile', 'Dockerfile.umbrella'):
            source = (ROOT / 'frontend' / name).read_text(encoding='utf8')
            stage = source.split('AS generated\n', 1)[1].split('\nFROM ', 1)[0]
            self.assertLess(stage.index('COPY wiki/frontend/scripts/base-generate-openapi.mjs'), stage.index('pnpm run postinstall'))

    def test_profile_seals_actual_commands_and_all_prepared_consumers(self):
        selector = runpy.run_path(str(ROOT / "scripts/vendor/base_quality_selection.py"))
        manifest = json.loads((ROOT / ".quality-gate.json").read_text(encoding="utf-8"))
        package = json.loads((ROOT / "frontend/package.json").read_text(encoding="utf-8"))
        profile = manifest["frontend_generation"]
        self.assertEqual(profile["schema"], "base-frontend-generation/v1")
        self.assertEqual(profile["environment_input_contract"], "frontend-agent-presence-v1")
        self.assertEqual(profile["scripts"], ["postinstall"])
        commands = {name: package["scripts"].get(name)
                    for name in ("postinstall", "prepostinstall", "postpostinstall")}
        self.assertEqual(selector["digest"](commands), profile["commands_sha256"])
        self.assertEqual(set(profile["inputs"]), {
            "frontend/scripts/base-generate-openapi.mjs", "openapi/**",
            "frontend/scripts/generate-exports.mjs", "frontend/redocly.yaml",
            "frontend/redocly.yml", "frontend/.redocly.yaml", "frontend/.redocly.yml",
        })
        self.assertEqual(manifest["frontend_generated"],
                         ["src/api/generated.ts", "src/api/generated-exports.ts"])
        for gate in manifest["gates"]:
            if gate.get("prepare_frontend"):
                with self.subTest(gate=gate["id"]):
                    self.assertEqual(gate["descriptor_input_contract"], "prepared-frontend-generation-context-v1")
                    if gate.get("kind") == "artifact":
                        self.assertEqual(gate["artifact_input_contract"], "git-source-v1")

    @staticmethod
    def external_refs(value):
        pending, found = [value], []
        while pending:
            current = pending.pop()
            if isinstance(current, dict):
                if "$ref" in current and not str(current["$ref"]).startswith("#"):
                    found.append(current["$ref"])
                pending.extend(current.values())
            elif isinstance(current, list):
                pending.extend(current)
        return found

    def test_qualified_graph_has_no_unreviewed_external_or_config_consumers(self):
        schema = json.loads((ROOT / "openapi/openapi.json").read_text(encoding="utf-8"))
        self.assertEqual(self.external_refs(schema), [], "Requalify external schema consumers")
        for name in ("redocly.yaml", "redocly.yml", ".redocly.yaml", ".redocly.yml"):
            self.assertFalse((ROOT / "frontend" / name).exists(), "Requalify new Redocly config/plugins")
        helper = (ROOT / "frontend/scripts/generate-exports.mjs").read_text(encoding="utf-8")
        # Reviewed incremental writer from 6c6bc5d / the original Wiki3 port.
        # Its extra read is the declared generated output, only to suppress a
        # byte-identical write; schema/config dependencies stay unchanged.
        # This Source guard grants no native prepared-environment reuse.
        self.assertEqual(hashlib.sha256(helper.encode()).hexdigest(),
                         "c2649c33fd9df9cf4f413f8b4cca1ed49a77ec89e08100f3557c70fb3eb2cb44",
                         "Review changed helper reads/imports and requalify its closure")
        lock = (ROOT / "frontend/pnpm-lock.yaml").read_text(encoding="utf-8")
        self.assertRegex(lock, r"(?m)^      openapi-typescript:\n        specifier: [^\n]+\n        version: 7\.13\.0\(typescript@5\.9\.3\)$")

    def test_external_ref_guard_accepts_fragments_and_finds_nested_consumers(self):
        self.assertEqual(self.external_refs({"schema": [{"$ref": "#/components/schemas/Page"}]}), [])
        for reference in ("../external.json#/Page", "https://example.test/openapi.json#/Page"):
            with self.subTest(reference=reference):
                self.assertEqual(self.external_refs({"schema": [{"items": {"$ref": reference}}]}), [reference])



class PnpmLifecyclePolicyTests(unittest.TestCase):
    def test_esbuild_has_one_explicit_dependency_build_policy(self):
        from pathlib import Path
        root = Path(__file__).resolve().parents[2]
        lines = (root / 'frontend/pnpm-workspace.yaml').read_text(encoding='utf-8').splitlines()
        self.assertEqual(lines.count('allowBuilds:'), 1)
        body = []
        for line in lines[lines.index('allowBuilds:') + 1:]:
            if line and not line.startswith(' '):
                break
            if line.strip():
                body.append(line)
        self.assertEqual(body, ['  esbuild: true'])
        for legacy in ('onlyBuiltDependencies:', 'onlyBuiltDependenciesFile:',
                       'ignoredBuiltDependencies:', 'neverBuiltDependencies:',
                       'dangerouslyAllowAllBuilds:'):
            self.assertNotIn(legacy, lines)



class RustCiJobReuseTests(unittest.TestCase):
    @staticmethod
    def job(name):
        import re
        source = (ROOT / '.github/workflows/ci.yml').read_text(encoding='utf-8')
        match = re.search(r'^  ' + re.escape(name) + r':\n(.*?)(?=^  [a-zA-Z][\w-]*:|\Z)',
                          source, re.M | re.S)
        if match is None:
            raise AssertionError('Missing CI job: ' + name)
        return match.group(1)

    def test_one_workspace_owns_rust_preparation_and_all_original_checks(self):
        backend = self.job('backend')
        summary = self.job('minimum-rust')
        self.assertIn("needs: changes", backend)
        self.assertIn("if: needs.changes.outputs.backend == 'true'", backend)
        self.assertEqual(backend.count('uses: dtolnay/rust-toolchain@stable'), 1)
        self.assertEqual(backend.count('toolchain: 1.88.0'), 1)
        self.assertEqual(backend.count('uses: Swatinem/rust-cache@v2'), 1)
        self.assertEqual(backend.count('run: python3 ../scripts/vendor/base_rust_target_cache.py'), 1)
        self.assertEqual(backend.count('uses: actions/checkout@'), 2)
        self.assertNotIn('uses:', summary)
        self.assertNotIn('cargo ', summary)
        self.assertIn('needs: [changes, backend]', summary)
        self.assertIn("if: always() && needs.changes.outputs.backend == 'true'", summary)

    def test_minimum_check_keeps_scope_environment_and_unit_failure_order(self):
        backend = self.job('backend')
        source = (ROOT / '.github/workflows/ci.yml').read_text(encoding='utf-8')
        minimum = 'env -u SQLX_OFFLINE cargo check --locked --workspace --all-targets'
        self.assertEqual(source.count('cargo check --locked --workspace --all-targets'), 1)
        self.assertIn(minimum, backend)
        self.assertLess(backend.index('cargo test --locked'), backend.index(minimum))
        self.assertLess(backend.index(minimum), backend.index('run: cargo build --locked'))
        self.assertIn("if: needs.changes.outputs.backend_build == 'true'", backend)
        self.assertIn('SQLX_OFFLINE: "true"', backend)
        self.assertNotIn('continue-on-error:', backend)

    def test_legacy_status_rejects_failure_cancellation_skip_and_missing_result(self):
        import os
        import shutil
        import subprocess
        if os.name == 'nt':
            # Use Git Bash explicitly; never launch the Windows WSL bash shim.
            bash = Path(os.environ.get('ProgramFiles', 'C:/Program Files')) / 'Git/bin/bash.exe'
            if not bash.is_file():
                self.skipTest('Git Bash is unavailable; run this shell contract on Linux')
        else:
            bash = shutil.which('bash')
            if not bash:
                self.skipTest('Bash is unavailable')
        summary = self.job('minimum-rust')
        self.assertIn('shell: bash', summary)
        self.assertIn('BACKEND_RESULT: ${{ needs.backend.result }}', summary)
        commands = [line.strip()[5:] for line in summary.splitlines()
                    if line.strip().startswith('run: ')]
        self.assertEqual(len(commands), 1)
        for result in ('success', 'failure', 'cancelled', 'skipped', '', 'unknown'):
            with self.subTest(result=result):
                process = subprocess.run([str(bash), '--noprofile', '--norc', '-c', commands[0]],
                    env={**os.environ, 'BACKEND_RESULT': result}, capture_output=True, timeout=10)
                self.assertEqual(process.returncode == 0, result == 'success')

if __name__ == "__main__":
    unittest.main()
