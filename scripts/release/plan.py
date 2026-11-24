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
    if version is None:
        raise ValueError(f"{commit}: set the workspace version to MAJOR.MINOR.PATCH")
    return tuple(map(int, version.groups()))


def tag_name(version):
    return "v" + ".".join(map(str, version))


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
    latest = max(versions)
    previous = tag_name(latest)
    declared = declared_version(previous)
    commits = git("rev-list", "--reverse", "--first-parent", "HEAD", f"^{previous}").splitlines()
    changes = [(commit, declared_version(commit)) for commit in commits]
    declared_bumps = iter(sorted({version for _, version in changes if version > declared}))
    next_release = next(declared_bumps, None)
    tags = {}
    for commit, version in changes:
        if version < declared:
            # Main can have commits predating the merge of the initial version.
            if declared[:2] == (0, 2) and not is_ancestor(BASELINE, commit):
                continue
            raise ValueError(f"{commit}: the workspace version decreased")

        if version > declared:
            tag = tag_name(version)
            if tag in known:
                raise ValueError(f"{tag} already belongs to another commit")
            if version <= latest:
                raise ValueError(f"{commit}: the workspace version must exceed {tag_name(latest)}")
            latest = declared = version
            next_release = next(declared_bumps, None)
        else:
            major, minor, patch = latest
            candidate = (major, minor, patch + 1)
            while tag_name(candidate) in known:
                candidate = (major, minor, candidate[2] + 1)
            # Leave the next declared version available for the commit that declares it.
            if next_release is not None and candidate >= next_release:
                continue
            latest = candidate
        tags[tag_name(latest)] = commit

    release_tag = tag_name(declared)
    release = None
    if release_tag != BASELINE:
        commit = tags.get(release_tag) or git("rev-parse", f"{release_tag}^{{commit}}")
        release = {"version": release_tag.removeprefix("v"), "commit": commit}
    return {"tags": tags, "release": release}


def is_ancestor(parent, commit):
    return subprocess.run(["git", "merge-base", "--is-ancestor", parent, commit]).returncode == 0


if __name__ == "__main__":
    try:
        print(json.dumps(plan()))
    except (ValueError, KeyError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
