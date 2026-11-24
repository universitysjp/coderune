import json
from pathlib import Path
import subprocess
import tempfile
import unittest

PLANNER = Path(__file__).with_name("plan.py").resolve()


class ReleasePlanTest(unittest.TestCase):
    def setUp(self):
        repository = tempfile.TemporaryDirectory()
        self.addCleanup(repository.cleanup)
        self.root = Path(repository.name)
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.email", "test@example.com")
        self.git("config", "user.name", "Release Test")
        self.commit("0.2.0")
        self.git("tag", "v0.2.0")

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, text=True).strip()

    def commit(self, version):
        (self.root / "Cargo.toml").write_text(
            f'[workspace.package]\nversion = "{version}"\n'
        )
        self.git("add", "Cargo.toml")
        self.git("commit", "-qm", f"Version {version}", "--allow-empty")
        return self.git("rev-parse", "HEAD")

    def plan(self):
        return json.loads(
            subprocess.check_output(["python3", str(PLANNER)], cwd=self.root, text=True)
        )

    def test_patch_push_tags_each_commit_without_a_build(self):
        first = self.commit("0.2.0")
        second = self.commit("0.2.0")
        self.assertEqual(
            self.plan(),
            {"tags": {"v0.2.1": first, "v0.2.2": second}, "release": None},
        )

    def test_manual_bump_builds_the_milestone_even_if_patches_follow(self):
        milestone = self.commit("0.3.0")
        patch = self.commit("0.3.0")
        release = {"version": "0.3.0", "commit": milestone}
        self.assertEqual(
            self.plan(),
            {"tags": {"v0.3.0": milestone, "v0.3.1": patch}, "release": release},
        )
        self.git("tag", "v0.3.0", milestone)
        self.git("tag", "v0.3.1", patch)
        self.assertEqual(self.plan(), {"tags": {}, "release": release})

    def test_explicit_patch_bump_reserves_its_tag_and_retries_its_build(self):
        self.git("tag", "v0.3.0", self.commit("0.3.0"))
        self.commit("0.3.0")
        release_commit = self.commit("0.3.1")
        later = self.commit("0.3.1")
        release = {"version": "0.3.1", "commit": release_commit}

        self.assertEqual(
            self.plan(),
            {"tags": {"v0.3.1": release_commit, "v0.3.2": later}, "release": release},
        )
        self.git("tag", "v0.3.1", release_commit)
        self.git("tag", "v0.3.2", later)
        self.assertEqual(self.plan(), {"tags": {}, "release": release})

    def test_explicit_patch_bump_cannot_reuse_an_automatic_tag(self):
        self.git("tag", "v0.3.0", self.commit("0.3.0"))
        automatic_tag = self.commit("0.3.0")
        self.git("tag", "v0.3.1", automatic_tag)
        self.commit("0.3.1")

        result = subprocess.run(
            ["python3", str(PLANNER)], cwd=self.root, capture_output=True, text=True
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("v0.3.1 already belongs to another commit", result.stderr)
        self.assertEqual(self.git("rev-parse", "v0.3.1"), automatic_tag)

    def test_merge_preserves_existing_tags_and_tags_the_merge(self):
        self.git("switch", "-qc", "setup")
        self.commit("0.2.0")
        self.git("tag", "v0.2.1")
        self.git("switch", "-q", "main")
        self.git("merge", "-q", "--no-ff", "setup")
        self.assertEqual(
            self.plan(),
            {"tags": {"v0.2.2": self.git("rev-parse", "HEAD")}, "release": None},
        )


if __name__ == "__main__":
    unittest.main()
