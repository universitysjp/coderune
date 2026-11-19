"""Plan the first-parent tags and the current desktop release from Git history."""

import json
import re
import subprocess
import sys
import tomllib

BASELINE = "v0.2.0"
VERSION = re.compile(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)")


def git(*args):
    return subprocess.check_output(["git", *args], text=True).strip()


def declared_version(commit):
    value = tomllib.loads(git("show", f"{commit}:Cargo.toml"))["workspace"]["package"]["version"]
    version = VERSION.fullmatch(value)
    if version is None or version[3] != "0":
        raise ValueError(f"{commit}: set the workspace version to MAJOR.MINOR.0")
    return int(version[1]), int(version[2])


def plan():
    known = set(git("tag", "--list").splitlines())
    merged = git("tag", "--merged", "HEAD").splitlines()
    if BASELINE not in merged:
        raise ValueError(f"Merge the commit tagged {BASELINE} into main before tagging releases")

    versions = [
        tuple(map(int, match.groups()))
        for tag in merged
        if (match := VERSION.fullmatch(tag.removeprefix("v"))) and tag.startswith("v")
    ]
    major, minor, patch = max(versions)
    previous = f"v{major}.{minor}.{patch}"
    tags = {}
    for commit in git("rev-list", "--reverse", "--first-parent", "HEAD", f"^{previous}").splitlines():
        line = declared_version(commit)
        if line < (major, minor):
            # Main can have commits predating the merge of the initial version.
            if (major, minor) == (0, 2) and not is_ancestor(BASELINE, commit):
                continue
            raise ValueError(f"{commit}: the workspace version decreased")
        if line == (major, minor):
            patch += 1
            while f"v{major}.{minor}.{patch}" in known:
                patch += 1
        else:
            major, minor = line
            patch = 0
        tag = f"v{major}.{minor}.{patch}"
        if tag in known:
            raise ValueError(f"{tag} already belongs to another commit")
        tags[tag] = commit

    version = f"{major}.{minor}.0"
    milestone = f"v{version}"
    release = None
    if milestone != BASELINE:
        commit = tags.get(milestone) or git("rev-parse", f"{milestone}^{{commit}}")
        release = {"version": version, "commit": commit}
    return {"tags": tags, "release": release}


def is_ancestor(parent, commit):
    return subprocess.run(["git", "merge-base", "--is-ancestor", parent, commit]).returncode == 0


if __name__ == "__main__":
    try:
        print(json.dumps(plan()))
    except (ValueError, KeyError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
