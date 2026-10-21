"""Wiki scoped plans use the packaged Base selector; release keeps full acceptance."""
import hashlib
import json
import re
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.vendor import base_quality_selection as selector

ROOT = Path(__file__).resolve().parents[2]
PIN = "81467ded7038631691566182c6cfc1106fcd7e84"


def job(text, name):
    return re.search(r"^  " + name + r":\n.*?(?=^  [a-zA-Z0-9_-]+:\n|\Z)", text, re.M | re.S)[0]


class QualityRouteTests(unittest.TestCase):
    def test_every_automatic_consumer_uses_the_selected_exact_head(self):
        ci = (ROOT/'.github/workflows/ci.yml').read_text(encoding='utf-8')
        paths = re.findall(r'^          path: wiki\n(.*?)(?=^      -|^  [a-zA-Z0-9_-]+:|\Z)', ci, re.M | re.S)
        self.assertEqual(len(paths), 4)
        for checkout in paths:
            self.assertIn('ref: ${{ github.event.pull_request.head.sha || github.sha }}', checkout)

    def plan(self, paths, profile="build", **options):
        with patch.object(selector, "git", return_value="a" * 40):
            return selector.plan(ROOT, profile, changed=paths, **options)

    def test_docs_and_no_change_do_not_select_application_producers(self):
        self.assertEqual(self.plan([])["gates"], [])
        for profile in ("fast", "unit", "build"):
            gates = self.plan(["docs/ARCHITECTURE.md"], profile)["gates"]
            self.assertFalse(any(g["component"] in ("frontend", "backend") for g in gates))
        self.assertEqual(self.plan(["scripts/tests/test_quality_routes.py"])["components"], ["tooling"])

    def test_frontend_source_and_lock_select_only_frontend(self):
        for path in ("frontend/src/pages/document/document.tsx", "frontend/pnpm-lock.yaml", ".base-ui-revision"):
            with self.subTest(path=path):
                selected = self.plan([path])
                self.assertEqual(selected["components"], ["frontend"])
                self.assertIn("frontend:build", [g["id"] for g in selected["gates"]])

    def test_frontend_tests_and_setup_keep_unit_static_without_vite(self):
        for path in ("frontend/src/pages/document/document.test.tsx", "frontend/src/test/setup.ts", "frontend/vitest.config.ts"):
            with self.subTest(path=path):
                ids = {g["id"] for g in self.plan([path])["gates"]}
                self.assertTrue({"frontend:tests", "frontend:types"} <= ids)
                self.assertNotIn("frontend:build", ids)

    def test_backend_and_sdk_remain_conservative_and_independent(self):
        manifest = selector.load(ROOT)
        self.assertNotIn("backend_unit_inputs", manifest)
        for path in ("backend/domain/src/lib.rs", "backend/infra/tests/postgres.rs", ".base-revision"):
            selected = self.plan([path])
            self.assertEqual(selected["components"], ["backend"])
            build = next(g for g in selected["gates"] if g["id"] == "backend:build")
            self.assertNotIn("exclude_inputs", build)
            self.assertEqual(build["sdk_revision"], ".base-revision")
        self.assertNotIn("native_execution", manifest)
        self.assertTrue(all(g["cacheable"] is False for g in manifest["gates"] if g.get("kind") == "artifact"))

    def test_service_artifact_names_only_the_actual_runtime_binary(self):
        gate = next(g for g in selector.load(ROOT)['gates'] if g['id'] == 'backend:build')
        self.assertEqual(gate['artifact_outputs'], ['server'])
        self.assertIn('cargo build --locked --release -p server --bin server', gate['argv'][-1])
        self.assertNotIn('--workspace', gate['argv'][-1])
        for name in ('Dockerfile', 'Dockerfile.umbrella'):
            self.assertIn('/app/target/release/server', (ROOT/'backend'/name).read_text(encoding='utf-8'))

    def test_unknown_and_explicit_full_selection_keep_application_builds(self):
        self.assertEqual(set(self.plan(["new-runtime-input.conf"])["components"]), {"docs", "frontend", "backend", "tooling"})
        explicit = self.plan([], components=["docs", "frontend", "backend", "tooling"])
        self.assertTrue({"frontend:build", "backend:build"} <= {g["id"] for g in explicit["gates"]})

    def test_domain_unit_is_fast_and_full_workspace_is_manual(self):
        native = [g for g in self.plan(["backend/domain/src/lib.rs"], "unit")["gates"] if g["component"] == "backend"]
        self.assertEqual(len(native), 1)
        self.assertIn("cargo test --locked -p domain --lib", native[0]["argv"][-1])
        self.assertNotIn("--workspace", native[0]["argv"][-1])
        full = next(g for g in self.plan(["backend/domain/src/lib.rs"], "release")["gates"] if g["id"] == "backend:quality")
        self.assertEqual(full["requires_env"], ["WIKI_TEST_DATABASE_URL"])
        self.assertIn("cargo test --locked --workspace -- --test-threads=1", full["argv"][-1])

    def test_actual_ci_selector_omits_only_frontend_test_producer(self):
        for path, build in (("frontend/src/pages/document/document.test.tsx", "false"), ("frontend/src/test/setup.ts", "false"), ("frontend/src/pages/document/document.tsx", "true")):
            with patch.object(selector, "git", return_value=path):
                outputs = selector.ci_outputs("pull_request", "a" * 40, "b" * 40, repo=ROOT)
            self.assertEqual(outputs, {"backend": "false", "frontend": "true", "frontend_build": build, "backend_build": "false"})
        with patch.object(selector, "git", return_value="backend/infra/tests/postgres.rs"):
            outputs = selector.ci_outputs("push", "a" * 40, "b" * 40, repo=ROOT)
        self.assertEqual(outputs["backend_build"], "true")

    def test_selector_packet_is_exact_and_has_no_optional_features(self):
        receipt = json.loads((ROOT / "scripts/vendor/base-quality-tooling.json").read_text(encoding="utf-8"))
        self.assertEqual((ROOT / ".quality-tooling-revision").read_text(encoding="utf-8").strip(), PIN)
        self.assertEqual(receipt["revision"], PIN)
        self.assertEqual(receipt["sha256"], hashlib.sha256((ROOT / "scripts/vendor/base_quality_selection.py").read_bytes()).hexdigest())
        self.assertEqual(set(receipt), {"schema", "repository", "revision", "source", "sha256"})
        changes = job((ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8"), "changes")
        code = re.search(r"          python3 - <<'PY'\n(.*?)^          PY$", changes, re.M | re.S)[1]
        code = "\n".join(line[10:] for line in code.splitlines())
        with patch("pathlib.Path", side_effect=lambda path: ROOT if str(path) == "wiki" else Path(path)):
            exec(compile(code, "<wiki-ci-selector-verifier>", "exec"), {})
        self.assertIn("git rev-parse HEAD", changes)
        self.assertIn("--no-tags --depth=1", changes)

    def test_auto_ci_has_one_rust_setup_and_retains_contract_checks(self):
        ci = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        self.assertEqual(ci.count("uses: dtolnay/rust-toolchain@stable"), 1)
        self.assertEqual(ci.count("uses: Swatinem/rust-cache@v2"), 1)
        for excluded in ("services:", "cargo test --locked --workspace", "--all-targets", "playwright install", "\n  minimum-rust:"):
            self.assertNotIn(excluded, ci)
        backend = job(ci, "backend")
        self.assertIn("cargo check --locked -p server --bin server", backend)
        self.assertIn("cargo run --locked -p api --bin openapi-gen", backend)
        self.assertIn("if: needs.changes.outputs.backend_build == 'true'", backend)
        frontend = job(ci, "frontend")
        for command in ("pnpm openapi:check", "pnpm typecheck", "pnpm test:unit", "pnpm lint", "pnpm format:check", "pnpm openapi:compat", "git diff --exit-code"):
            self.assertIn(command, frontend)
        self.assertIn("if: needs.changes.outputs.frontend_build == 'true'", frontend)

    def test_manual_release_keeps_original_obligations(self):
        release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        self.assertEqual(release.split("\non:\n", 1)[1].split("\n\n", 1)[0].strip(), "workflow_dispatch:")
        self.assertEqual(set(re.findall(r"^  ([a-zA-Z0-9_-]+):$", release.split("jobs:\n", 1)[1], re.M)), {"docs", "backend", "frontend", "minimum-rust"})
        for command in ("cargo clippy --locked --workspace --all-targets -- -D warnings", "cargo check --locked --workspace --all-targets", "cargo test --locked --workspace -- --test-threads=1", "mixed_central_directory_preserves_identity_and_disabled_state_in_postgres -- --ignored --test-threads=1", "pnpm test:all", "python ../../services-base/scripts/check_ui_consumer.py", "pnpm theme:check http://127.0.0.1:4173"):
            self.assertIn(command, release)
        self.assertNotIn("run: pnpm build", release)
        self.assertIn("node scripts/generate-exports.mjs && pnpm exec tsc --noEmit && pnpm exec vite build", release)

    def test_ui_pin_owner_and_single_unit_run_are_explicit(self):
        self.assertEqual((ROOT / ".base-ui-revision").read_text(encoding="utf-8"), (ROOT / ".base-revision").read_text(encoding="utf-8"))
        for name in ("ci.yml", "release.yml"):
            text = (ROOT / ".github/workflows" / name).read_text(encoding="utf-8")
            self.assertIn("cat .base-ui-revision", job(text, "frontend"))
            self.assertIn("--revision .base-ui-revision", job(text, "frontend"))
            self.assertNotIn("cat .base-revision", job(text, "frontend"))
            self.assertIn("cat .base-revision", job(text, "backend"))
        scripts = json.loads((ROOT / "frontend/package.json").read_text(encoding="utf-8"))["scripts"]
        self.assertEqual(scripts["build"].count("test:unit"), 1)
        self.assertEqual(scripts["test:all"], "vitest run")
        for tier in ("integration", "slow", "e2e"):
            self.assertIn("**/*." + tier + ".test.*", scripts["test:unit"])
        self.assertIn("/.local/", (ROOT / ".gitignore").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
