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
