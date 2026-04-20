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

    def test_tauri_cargo_manifest_defines_custom_protocol_feature(self):
        cargo_toml = (REPO_ROOT / 'src-tauri' / 'Cargo.toml').read_text(encoding='utf-8')
        self.assertRegex(cargo_toml, r'(?ms)^\[features\]\s+.*^custom-protocol\s*=\s*\[\s*"tauri/custom-protocol"\s*\]')

    def test_activation_import_does_not_depend_on_runtime_current_dir(self):
        main_rs = (REPO_ROOT / 'src-tauri' / 'src' / 'main.rs').read_text(encoding='utf-8')
        self.assertNotIn('std::env::current_dir()', main_rs)
        self.assertIn('include_str!("../../keys/license_priv.pem")', main_rs)

    def test_windows_icon_exists_for_tauri_bundle(self):
        self.assertTrue((REPO_ROOT / 'src-tauri' / 'icons' / 'icon.ico').exists())

    def test_tauri_bundle_config_references_ico_icon(self):
        config = json.loads(TAURI_CONFIG_PATH.read_text(encoding='utf-8'))
        icons = config['tauri']['bundle'].get('icon', [])
        self.assertIn('icons/icon.ico', icons)

    def test_mac_icon_exists_for_tauri_bundle(self):
        self.assertTrue((REPO_ROOT / 'src-tauri' / 'icons' / 'icon.icns').exists())

    def test_tauri_bundle_config_references_icns_icon(self):
        config = json.loads(TAURI_CONFIG_PATH.read_text(encoding='utf-8'))
        icons = config['tauri']['bundle'].get('icon', [])
        self.assertIn('icons/icon.icns', icons)

    def test_hide_layout_updates_lumina_structure(self):
        inject_js = (REPO_ROOT / 'src-tauri' / 'src' / 'inject.js').read_text(encoding='utf-8')
        self.assertIn("document.querySelectorAll('.ba-sider').forEach((el) => {", inject_js)
        self.assertIn("el.style.width = '0';", inject_js)
        self.assertIn("const wrapper = document.querySelector('.ba-generate-framework-main-content-wrapper');", inject_js)
        self.assertIn("wrapper.classList.remove('flex-none');", inject_js)
        self.assertIn("wrapper.classList.add('flex-1');", inject_js)
        self.assertIn("const first = wrapper.querySelector(':scope > div:first-child') || wrapper.firstElementChild;", inject_js)
        self.assertIn("first.style.width = '100%';", inject_js)
        self.assertIn("document.body.style.paddingTop = '0';", inject_js)
        self.assertIn("document.body.style.overflowY = 'hidden';", inject_js)

    def test_hide_layout_hides_second_and_third_framework_children(self):
        inject_js = (REPO_ROOT / 'src-tauri' / 'src' / 'inject.js').read_text(encoding='utf-8')
        self.assertIn("const framework = document.querySelector('.ba-generate-framework');", inject_js)
        self.assertIn("const second = framework.children[1];", inject_js)
        self.assertIn("const third = framework.children[2];", inject_js)
        self.assertIn("second.classList.add('hidden');", inject_js)
        self.assertIn("third.classList.add('hidden');", inject_js)
        self.assertIn("const toHide = ['智能视频', '自由创作'];", inject_js)

    def test_hide_layout_rewrites_lumina_headline(self):
        inject_js = (REPO_ROOT / 'src-tauri' / 'src' / 'inject.js').read_text(encoding='utf-8')
        self.assertIn("const h1 = document.querySelector('.model-experience-main-content h1')", inject_js)
        self.assertIn("h1.textContent = '开启您的创作之旅'", inject_js)

    def test_hide_layout_runs_during_tick_boot_and_runtime(self):
        inject_js = (REPO_ROOT / 'src-tauri' / 'src' / 'inject.js').read_text(encoding='utf-8')
        self.assertRegex(inject_js, r"window\.__sv_tick__\s*=\s*\(reason = 'external'\) => \{[\s\S]*?hideLayout\(\);")
        self.assertRegex(inject_js, r"const boot = async \(\) => \{[\s\S]*?hideLayout\(\);")
        self.assertRegex(inject_js, r"setInterval\(\(\) => \{[\s\S]*?hideLayout\(\);[\s\S]*?\}, 1200\)")

    def test_activation_modal_close_button_uses_centered_svg_icon(self):
        inject_js = (REPO_ROOT / 'src-tauri' / 'src' / 'inject.js').read_text(encoding='utf-8')
        self.assertIn('display:flex;align-items:center;justify-content:center;', inject_js)
        self.assertIn('.sv-close svg{width:14px;height:14px;display:block}', inject_js)
        self.assertIn('<svg', inject_js)
        self.assertIn('viewBox="0 0 24 24"', inject_js)
        self.assertNotIn('>×</button>', inject_js)


if __name__ == '__main__':
    unittest.main()
