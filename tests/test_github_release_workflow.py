import json
import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_PATH = REPO_ROOT / '.github' / 'workflows' / 'release.yml'
TAURI_CONFIG_PATH = REPO_ROOT / 'src-tauri' / 'tauri.conf.json'


class GitHubReleaseWorkflowTests(unittest.TestCase):
    def test_release_workflow_exists(self):
        self.assertTrue(WORKFLOW_PATH.exists(), 'expected .github/workflows/release.yml to exist')

    def test_workflow_builds_macos_and_windows(self):
        content = WORKFLOW_PATH.read_text(encoding='utf-8')
        self.assertIn("- 'v*'", content)
        self.assertIn('workflow_dispatch:', content)
        self.assertIn('aarch64-apple-darwin', content)
        self.assertIn('x86_64-pc-windows-msvc', content)
        self.assertRegex(content, r'cargo\s+install\s+tauri-cli')
        self.assertRegex(content, r'cargo\s+tauri\s+build\s+--target\s+\$\{\{\s*matrix\.target\s*\}\}')

    def test_workflow_uploads_release_artifacts(self):
        content = WORKFLOW_PATH.read_text(encoding='utf-8')
        self.assertIn('softprops/action-gh-release@v2', content)
        self.assertIn('artifacts/**/*.dmg', content)
        self.assertIn('artifacts/**/*.msi', content)
        self.assertIn('artifacts/**/*.exe', content)

    def test_tauri_bundle_is_enabled_for_packaging(self):
        config = json.loads(TAURI_CONFIG_PATH.read_text(encoding='utf-8'))
        self.assertTrue(config['tauri']['bundle']['active'])


if __name__ == '__main__':
    unittest.main()
