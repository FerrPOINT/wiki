"""Known pure Python tests select no application producer."""
import shlex
import unittest
from pathlib import Path
from unittest.mock import patch
from scripts.vendor import base_quality_selection as selection

ROOT = Path(__file__).resolve().parents[2]
TESTS = ('test_build_pin', 'test_verify_readme', 'test_tooling_ownership', 'test_build_scope')


class PythonToolingOwnershipTests(unittest.TestCase):
    def test_agent_presence_contract_loads_and_projects_only_types_and_units(self):
        gates = selection.load(ROOT)['gates']
        self.assertEqual({g['id']: g['environment_input_contract'] for g in gates
                          if 'environment_input_contract' in g}, {
            'frontend:types': 'frontend-agent-presence-v1',
            'frontend:tests': 'frontend-agent-presence-v1',
        })
        for gate in gates:
            values = {'CODEX_THREAD_ID': 'chat-a', 'CODEX_SESSION_ID': 'session-a',
                      'CODEX_SANDBOX': 'workspace', 'VITE_API_BASE_URL': '/api/v1',
                      'NODE_EXTRA_CA_CERTS': 'fixture.pem', 'HTTPS_PROXY': 'fixture-proxy',
                      'UNKNOWN_PRODUCT_INPUT': 'retained'}
            expected = dict(values)
            if gate['id'] in ('frontend:types', 'frontend:tests'):
                expected.update(CODEX_THREAD_ID='1', CODEX_SESSION_ID='1')
            with self.subTest(gate=gate['id']):
                self.assertEqual(selection.artifact_environment(gate, values), expected)
                empty = {'CODEX_THREAD_ID': '', 'VITE_API_BASE_URL': '/api/v1'}
                self.assertEqual(selection.artifact_environment(gate, empty), empty)

    def test_readme_validator_source_selects_docs_and_helpers_without_application_work(self):
        for profile in ('fast', 'unit', 'build'):
            with self.subTest(profile=profile):
                candidate = selection.plan(ROOT, profile=profile, changed=['scripts/verify_readme.py'])
                self.assertEqual(candidate['components'], ['docs', 'tooling'])
                expected = {'tooling:python-helpers', 'docs:readme'} if profile == 'fast' else {'tooling:python-helpers'}
                self.assertEqual({gate['id'] for gate in candidate['gates']}, expected)
                self.assertFalse(any(gate.get('kind') == 'artifact' or gate.get('prepare_frontend')
                                     for gate in candidate['gates']))

    def test_known_tests_select_only_python_helpers(self):
        manifest = selection.load(ROOT)
        for profile in ('fast', 'unit', 'build'):
            for name in TESTS:
                with self.subTest(profile=profile, name=name):
                    candidate = selection.plan(ROOT, profile=profile,
                        changed=['scripts/tests/' + name + '.py'], manifest=manifest)
                    self.assertEqual(candidate['components'], ['tooling'])
                    self.assertEqual([g['id'] for g in candidate['gates']], ['tooling:python-helpers'])
                    gate = candidate['gates'][0]
                    self.assertEqual(gate['argv'][0], '{python}')
                    self.assertFalse(gate.get('kind') == 'artifact')
                    self.assertTrue({'scripts.tests.' + n for n in TESTS} <= set(gate['argv']))

    def test_ci_readme_validator_diff_skips_producers_and_keeps_neighboring_backend_change(self):
        base, head = '1' * 40, '2' * 40
        for paths, backend in ((['scripts/verify_readme.py'], 'false'),
                               (['scripts/verify_readme.py', 'backend/server/src/main.rs'], 'true')):
            with self.subTest(paths=paths):
                def changed(root, *arguments):
                    self.assertEqual(arguments, ('diff', '--name-only', '--no-renames', base, head))
                    return '\n'.join(paths)
                with patch.object(selection, 'git', side_effect=changed):
                    result = selection.ci_outputs('pull_request', base, head, repo=ROOT)
                self.assertEqual(result['backend'], backend)
                self.assertEqual(result['backend_build'], backend)
                self.assertEqual(result['frontend'], 'false')
                self.assertEqual(result['frontend_build'], 'false')

    def test_contract_test_keeps_its_existing_gate_and_unknown_script_stays_wide(self):
        contract = selection.plan(ROOT, profile='build', changed=['scripts/tests/test_ci_contract_checks.py'])
        self.assertEqual([g['id'] for g in contract['gates']], ['tooling:build-contracts'])
        for path, owner in [('scripts/new_generator.py', None),
                            ('frontend/src/product.tsx', 'frontend'), ('backend/src/lib.rs', 'backend')]:
            with self.subTest(path=path):
                candidate = selection.plan(ROOT, profile='build', changed=[path])
                if owner:
                    self.assertIn(owner, candidate['components'])
                else:
                    self.assertTrue({'backend', 'frontend'} <= set(candidate['components']))
                self.assertTrue(any(g.get('kind') == 'artifact' for g in candidate['gates']))

    def test_ci_includes_each_tooling_module_once(self):
        ci = (ROOT / '.github/workflows/ci.yml').read_text(encoding='utf-8')
        commands = [shlex.split(line.strip().removeprefix('run: ')) for line in ci.splitlines()
                    if line.strip().removeprefix('run: ').startswith('python3 -m unittest ')]
        for name in (*TESTS, 'test_ci_contract_checks'):
            with self.subTest(module=name):
                self.assertEqual(sum(c.count('scripts.tests.' + name) for c in commands), 1)
        self.assertIn('run: python3 scripts/verify_readme.py', ci)


class CssInputOwnershipTests(unittest.TestCase):
    def setUp(self):
        import importlib.util
        from pathlib import Path
        from unittest.mock import patch
        self.root = Path(__file__).resolve().parents[2]
        spec = importlib.util.spec_from_file_location('css_input_selection', self.root / 'scripts/vendor/base_quality_selection.py')
        self.selection = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.selection)
        current = patch.object(self.selection, 'git', return_value='a' * 40)
        current.start()
        self.addCleanup(current.stop)

    def test_css_fixture_is_unit_only_and_called_by_public_commands(self):
        import json
        manifest = self.selection.load(self.root)
        path = 'frontend/scripts/tailwind-source-inputs.test.mjs'
        build = next(g for g in manifest['gates'] if g['id'] == 'frontend:build')
        self.assertTrue(self.selection.matches(path, manifest['frontend_unit_inputs']))
        self.assertTrue(self.selection.matches(path, build['exclude_inputs']))
        package = json.loads((self.root / 'frontend/package.json').read_text(encoding='utf-8'))
        self.assertTrue(package['scripts']['test:unit'].startswith('node --test scripts/tailwind-source-inputs.test.mjs scripts/tests/generate-exports.test.mjs && vitest run '))
        self.assertEqual(package['scripts']['test:all'], 'node --test scripts/tailwind-source-inputs.test.mjs scripts/tests/generate-exports.test.mjs && vitest run')

    def test_css_fixture_skips_application_build_but_real_css_keeps_it(self):
        path = 'frontend/scripts/tailwind-source-inputs.test.mjs'
        for profile in ('fast', 'unit', 'build'):
            with self.subTest(profile=profile):
                plan = self.selection.plan(self.root, profile=profile, changed=[path])
                self.assertIn('frontend:tests', [g['id'] for g in plan['gates']])
                self.assertFalse(any(g.get('kind') == 'artifact' for g in plan['gates']))
        mixed = self.selection.plan(self.root, profile='build', changed=[path, 'frontend/src/index.css'])
        self.assertIn('frontend:build', [g['id'] for g in mixed['gates']])
        self.assertNotIn('backend', mixed['components'])

    def test_ci_css_fixture_preserves_production_and_backend_selection(self):
        from unittest.mock import patch
        path = 'frontend/scripts/tailwind-source-inputs.test.mjs'
        for changed, build in ((path+'\n', 'false'), (path+'\nfrontend/src/index.css\n', 'true')):
            with self.subTest(build=build), patch.object(self.selection, 'git', return_value=changed):
                values = self.selection.ci_outputs('push', 'a'*40, 'b'*40, repo=self.root)
            self.assertEqual(values, {'backend':'false','frontend':'true','backend_build':'false','frontend_build':build})


    def test_generator_controls_are_unit_only_and_keep_real_input_builds(self):
        import json
        from unittest.mock import patch
        path = 'frontend/scripts/tests/generate-exports.test.mjs'
        manifest = self.selection.load(self.root)
        build = next(g for g in manifest['gates'] if g['id'] == 'frontend:build')
        self.assertTrue(self.selection.matches(path, manifest['frontend_unit_inputs']))
        self.assertTrue(self.selection.matches(path, build['exclude_inputs']))
        package = json.loads((self.root / 'frontend/package.json').read_text(encoding='utf-8'))
        for name in ('test:unit', 'test:all'):
            self.assertIn('scripts/tests/generate-exports.test.mjs', package['scripts'][name])
        for profile in ('fast', 'unit', 'build'):
            plan = self.selection.plan(self.root, profile=profile, changed=[path])
            self.assertIn('frontend:tests', [gate['id'] for gate in plan['gates']])
            self.assertFalse(any(gate.get('kind') == 'artifact' for gate in plan['gates']))
        producer = 'frontend/scripts/generate-exports.mjs'
        mixed = self.selection.plan(self.root, profile='build', changed=[path, producer])
        self.assertIn('frontend:build', [gate['id'] for gate in mixed['gates']])
        self.assertNotIn('backend', mixed['components'])
        for changed, expected in ((path + '\n', 'false'), (path + '\n' + producer + '\n', 'true')):
            with patch.object(self.selection, 'git', return_value=changed):
                values = self.selection.ci_outputs('push', 'a' * 40, 'b' * 40, repo=self.root)
            self.assertEqual(values, {'backend': 'false', 'frontend': 'true',
                                      'backend_build': 'false', 'frontend_build': expected})


if __name__ == '__main__':
    unittest.main()
