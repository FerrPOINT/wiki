"""Known pure Python tests select no application producer."""
import shlex
import unittest
from pathlib import Path
from unittest.mock import patch
from scripts.vendor import base_quality_selection as selection

ROOT = Path(__file__).resolve().parents[2]
TESTS = ('test_build_pin', 'test_verify_readme', 'test_tooling_ownership')


class PythonToolingOwnershipTests(unittest.TestCase):
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
        for path, owner in [('scripts/new_generator.py', None), ('scripts/build.py', None),
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


if __name__ == '__main__':
    unittest.main()
