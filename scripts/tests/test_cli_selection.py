"""Product selection only: no preparation, compiler, Docker or application calls."""
import json
import re
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import select_ci
from scripts.vendor import base_quality_selection as selector

ROOT = Path(__file__).resolve().parents[2]


class CliSelectionTests(unittest.TestCase):
    def plan(self, paths, profile='build', components=None):
        with patch.object(selector, 'git', return_value='a' * 40):
            return selector.plan(ROOT, profile=profile, changed=paths, components=components)

    def artifacts(self, paths):
        return {gate['id'] for gate in self.plan(paths)['gates'] if gate.get('kind') == 'artifact'}

    def test_cli_formatter_needs_no_sdk_or_cargo_target(self):
        manifest = selector.load(ROOT)
        gate = next(gate for gate in manifest['gates'] if gate['id'] == 'cli:format')
        self.assertEqual(gate['argv'][:4], ['rustfmt', '--edition', '2024', '--check'])
        self.assertTrue(all(name.startswith('cli/src/') and name.endswith('.rs') for name in gate['argv'][4:]))
        self.assertNotIn('sdk_revision', gate)
        self.assertEqual(gate['test_tier'], 'static')
        self.assertEqual(gate['cwd'], 'backend')
        self.assertEqual(gate['platforms'], ['linux'])
        self.assertEqual(gate['tools'][0]['pattern'], r'^rustc 1\.88\.0\b')
        self.assertIn('sdk_revision', next(gate for gate in manifest['gates'] if gate['id'] == 'cli:build'))

    def test_only_actual_application_inputs_select_producers(self):
        cases = [([], set()), (['README.md'], set()),
                 (['backend/cli/src/main.rs'], {'cli:build'}),
                 (['backend/server/src/main.rs'], {'backend:build'}),
                 (['frontend/src/main.tsx'], {'frontend:build'}),
                 (['backend/cli/tests/http.rs'], set()),
                 (['frontend/src/api/client.test.ts'], set()),
                 (['backend/Cargo.lock'], {'backend:build', 'cli:build'}),
                 (['.base-revision'], {'backend:build', 'cli:build'})]
        for paths, expected in cases:
            with self.subTest(paths=paths):
                self.assertEqual(self.artifacts(paths), expected)

    def test_cli_static_checks_do_not_compile_server_or_run_http_integration(self):
        for profile in ('fast', 'unit'):
            gates = self.plan(['backend/cli/src/main.rs'], profile)['gates']
            self.assertEqual([gate['id'] for gate in gates], ['cli:format'])
            self.assertEqual(gates[0]['test_tier'], 'static')
        build = self.plan(['backend/cli/src/main.rs'])['gates']
        self.assertEqual({gate['id'] for gate in build}, {'cli:format', 'cli:build'})
        producer = next(gate for gate in build if gate['id'] == 'cli:build')
        self.assertIn('-p wiki-cli --bin wiki', producer['argv'][-1])
        self.assertNotIn('--workspace', producer['argv'][-1])
        self.assertFalse(producer['cacheable'])

    def test_manual_cli_suite_keeps_the_full_workspace_and_runs_once_for_both(self):
        manifest = selector.load(ROOT)
        original = next(gate for gate in manifest['gates'] if gate['id'] == 'backend:quality')
        cli = self.plan([], 'release', ['cli'])['gates']
        self.assertEqual(len(cli), 1)
        self.assertEqual(cli[0]['argv'], original['argv'])
        self.assertFalse(cli[0]['cacheable'])
        combined = self.plan([], 'release', ['backend', 'cli'])['gates']
        manual = [gate for gate in combined if gate.get('shared_check')]
        self.assertEqual(len(manual), 1)
        self.assertEqual({row['component'] for row in manual[0]['shared_check_members']}, {'backend', 'cli'})

    def test_ci_uses_the_product_plan_and_invalid_base_stops_before_preparation(self):
        def git(_root, *args):
            if args == ('rev-parse', 'HEAD'):
                return 'b' * 40
            if args == ('diff', '--name-only', '--no-renames', 'a' * 40, 'b' * 40):
                return 'backend/cli/src/main.rs\n'
            raise AssertionError(args)
        with patch.object(selector, 'git', side_effect=git):
            output = select_ci.outputs(ROOT, 'pull_request', 'a' * 40, 'b' * 40)
        self.assertTrue(output['backend'])
        self.assertTrue(output['cli_build'])
        self.assertFalse(output['server_checks'])
        self.assertFalse(output['backend_build'])
        self.assertFalse(output['frontend'])
        with patch.object(selector, 'git') as calls:
            with self.assertRaises(ValueError):
                select_ci.outputs(ROOT, 'pull_request', '', 'b' * 40)
            calls.assert_not_called()

    def test_cli_binary_is_not_a_container_service(self):
        plan = self.plan(['backend/cli/src/main.rs'])
        path = ROOT / 'scripts/select_build.py'
        if path.exists():
            from scripts import select_build
            self.assertEqual(select_build.build_services(plan, selector.load(ROOT)), [])
        else:
            self.assertEqual(select_ci.build_services(plan), [])

    def test_ci_checkouts_consume_the_same_exact_candidate_as_selection(self):
        workflow = (ROOT / '.github/workflows/ci.yml').read_text(encoding='utf-8')
        checkouts = re.findall(r'^          path: wiki\n(.*?)(?=^      -|^  [a-zA-Z0-9_-]+:|\Z)',
                               workflow, re.MULTILINE | re.DOTALL)
        self.assertEqual(len(checkouts), 4)
        for checkout in checkouts:
            self.assertIn('ref: ${{ github.event.pull_request.head.sha || github.sha }}', checkout)

    def test_ci_declares_and_consumes_cli_and_server_selection(self):
        workflow = (ROOT / '.github/workflows/ci.yml').read_text(encoding='utf-8')
        self.assertIn('cli_build: ${{ steps.classify.outputs.cli_build }}', workflow)
        self.assertIn('server_checks: ${{ steps.classify.outputs.server_checks }}', workflow)
        self.assertTrue('python3 wiki/scripts/select_ci.py' in workflow
                        or 'python3 -m scripts.select_ci' in workflow)
        self.assertIn("if: needs.changes.outputs.cli_build == 'true'", workflow)
        self.assertIn("if: needs.changes.outputs.server_checks == 'true'", workflow)
        self.assertIn('cargo check --locked -p wiki-cli --bin wiki', workflow)
        self.assertIn('scripts.tests.test_cli_selection', workflow)
        manifest = json.loads((ROOT / '.quality-gate.json').read_bytes())
        owners = [gate for gate in manifest['gates'] if 'scripts.tests.test_cli_selection' in gate['argv']]
        self.assertEqual(len(owners), 1)
        self.assertIn('scripts/tests/test_cli_selection.py', owners[0]['inputs'])


if __name__ == '__main__':
    unittest.main()
