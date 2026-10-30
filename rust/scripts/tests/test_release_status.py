"""Regression tests for the crates.io release decision (no registry credentials)."""
import importlib.util
import io
import json
import os
import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch
from urllib.error import HTTPError, URLError

SCRIPT = Path(__file__).resolve().parents[1] / 'release-status.py'
spec = importlib.util.spec_from_file_location('release_status', SCRIPT)
release_status = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release_status)


class ReleaseStatusTests(unittest.TestCase):
    def check_index(self, body):
        with patch.object(release_status, 'urlopen', return_value=io.BytesIO(body)) as request:
            result = release_status.needs_publish('grammar-expressions', '0.1.1')
        self.assertEqual(request.call_args.args[0],
                         'https://index.crates.io/gr/am/grammar-expressions')
        self.assertEqual(request.call_args.kwargs['timeout'], 30)
        return result

    def test_unpublished_crate(self):
        error = HTTPError('url', 404, 'Not Found', {}, None)
        with patch.object(release_status, 'urlopen', side_effect=error):
            self.assertTrue(release_status.needs_publish('grammar-expressions', '0.1.1'))

    def test_new_version(self):
        self.assertTrue(self.check_index(b'{"vers":"0.1.0"}\n'))

    def test_existing_version(self):
        self.assertFalse(self.check_index(b'{"vers":"0.1.0"}\n{"vers":"0.1.1"}\n'))

    def test_yanked_version_cannot_be_republished(self):
        self.assertFalse(self.check_index(b'{"vers":"0.1.1","yanked":true}\n'))

    def test_registry_errors_fail_instead_of_publishing(self):
        for code in (401, 403, 429, 500):
            with self.subTest(code=code):
                error = HTTPError('url', code, 'Registry error', {}, None)
                with patch.object(release_status, 'urlopen', side_effect=error):
                    with self.assertRaises(HTTPError):
                        release_status.needs_publish('grammar-expressions', '0.1.1')

    def test_network_errors_fail(self):
        with patch.object(release_status, 'urlopen', side_effect=URLError('offline')):
            with self.assertRaises(URLError):
                release_status.needs_publish('grammar-expressions', '0.1.1')

    def test_invalid_index_fails(self):
        with self.assertRaises(json.JSONDecodeError):
            self.check_index(b'not json')

    def test_main_emits_github_output_from_manifest(self):
        manifest = SCRIPT.parents[1] / 'Cargo.toml'
        package = release_status.tomllib.loads(manifest.read_text())['package']
        for decision in (True, False):
            with self.subTest(publish=decision), tempfile.TemporaryDirectory() as directory:
                output = Path(directory) / 'output'
                with patch.dict(os.environ, {'GITHUB_OUTPUT': str(output)}):
                    with patch.object(release_status, 'needs_publish', return_value=decision) as check:
                        with patch('sys.stdout', new_callable=io.StringIO):
                            release_status.main()
                check.assert_called_once_with(package['name'], package['version'])
                self.assertEqual(output.read_text(), f'publish={str(decision).lower()}\n')

    def test_empty_index_fails(self):
        with self.assertRaises(ValueError):
            self.check_index(b'')


if __name__ == '__main__':
    unittest.main()
