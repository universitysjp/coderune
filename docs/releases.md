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

A push to `main` tags each new first-parent commit. For example, changes
in the 0.2 series receive `v0.2.1`, `v0.2.2`, and so on. Patches do not
start a desktop build. Tag and release jobs run serially, and the planner
catches up if multiple pushes are queued.

To publish a new release, change the workspace version in `Cargo.toml` to
the next major or minor line with a zero patch, for example `0.3.0`. Run
`cargo metadata --format-version 1 > /dev/null` after setting up Rune to
refresh `Cargo.lock`. Commit both changes. A labeled PR can test the packages
first. Once the bump reaches `main`, CI tags the milestone commit, builds and
tests it on all three systems, and publishes the packages and SHA-256 checksums
on [GitHub Releases](https://github.com/universitysjp/coderune/releases).

The latest unpublished milestone can be retried by rerunning the failed jobs
or on the next push to `main`. When more than one version line lands in a
single push, CI publishes the newest milestone. Published releases are left
intact.

## Update an installed copy

Open Settings and choose **Check for updates** under **Application
updates**. The app shows its installed version and offers the matching
package if GitHub has a newer published release.

Update checks and downloads use public GitHub Releases.

Choose **Download Coderune**. The app verifies the downloaded bytes against
the release asset's SHA-256 digest. Open the download folder, close the
running app, and follow the install step for your platform in the table
above. Your workspace database lives outside the application install.
