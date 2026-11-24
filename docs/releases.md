# Desktop builds and releases

Add `ready-to-build` to a pull request targeting `main`. GitHub Actions tests
the Rust workspace and uploads packages for the pull request head on Linux
x64, macOS ARM64, and Windows x64. Push another revision while the label is
present to rebuild. Download the artifact for your system from the workflow
run summary and extract its outer GitHub Actions ZIP.

| Platform | Package | Install |
| --- | --- | --- |
| Ubuntu 24.04 or a compatible x64 distribution | `.deb` | Open the package in your software installer |
| Apple Silicon macOS | `.zip` | Extract and move Coderune to Applications |
| Windows x64 | `-setup.exe` | Run the per-user installer and launch Coderune from Start |

The packages include the Coderune icon. Test builds use a version tag on the
exact commit when one exists. Otherwise they use the workspace version with a
pull request prerelease suffix. macOS builds have an ad hoc signature and
Windows installers are unsigned, so system policies may restrict installation.

## Tags and published releases

Use a merge commit to land the initial release setup so its version tags
remain in the history of `main`. The baseline is `v0.2.0`.

A push to `main` assigns sequential tags to new first-parent commits. An
explicit workspace version bump reserves its exact tag, so earlier commits in
the same push receive tags only where numbers are available. Automatic tags
identify commits. An explicit version bump starts the desktop build. Tag and
release jobs run serially, and the planner catches up if multiple pushes are
queued.

To publish a new release, choose a version above the latest tag and set the
workspace version in `Cargo.toml`, for example `0.3.1`. Run
`cargo metadata --format-version 1 > /dev/null` after setting up Rune to
refresh `Cargo.lock`. Commit both changes. A labeled PR can test the packages
first. Once the bump reaches `main`, CI tags the release commit, builds and
tests it on all three systems, and publishes the packages and SHA-256 checksums
on [GitHub Releases](https://github.com/universitysjp/coderune/releases).

The latest unpublished release can be retried by rerunning the failed jobs or
on the next push to `main`. When more than one explicit version lands in a
single push, CI publishes the newest.

## Update an installed copy

Open **Settings → General** and choose **Check for updates**. The app
shows its installed version and offers the matching
package if GitHub has a newer published release.

General also has a Dark mode switch. Settings remembers that choice across
launches. In **Settings → Agents**, use the switch beside an installed coding
agent to turn it on or off for Coderune.

Update checks and downloads use public GitHub Releases.

Choose **Download Coderune**. The app verifies the downloaded bytes against
the release asset's SHA-256 digest, then installs the update in place:

- **macOS** unpacks the new `Coderune.app` beside the installed one, checks its
  identifier, version, and code signature, and swaps the two bundles. A
  read-only or translocated copy keeps the package in Downloads instead.
- **Windows** runs the signed per-user installer silently over the current
  install, then reopens Coderune.
- **Linux** keeps the `.deb` in Downloads for `sudo dpkg -i`, because package
  installs belong to the system package manager.

The button becomes **Restart to update**: restarting quits the old version,
finishes the swap, and reopens the new one. The old bundle is removed on the
next launch. Your workspace database lives outside the application install, so
projects, captures, and settings survive the update.
