# Packaging and installation

For end users, the recommended path is the Windows x64 MSI from the
[latest GitHub Release](https://github.com/prluigileoni-png/Magic-pointer-cross-platform/releases/latest).
Install it, then launch **Magic Pointer** from the Start Menu. The graphical
first-run wizard asks for the API key, provider, and model. The Settings page
later allows these values and capture preferences to be changed without
editing JSON manually.

The CI workflow builds reproducible release binaries for all requested
architectures. Windows installers should package the `.exe` together with
`config.example.json`; the macOS release should be placed in a signed,
notarized `.app` bundle. Signing certificates and notarization credentials
must be supplied by the release environment and are intentionally not stored
in this repository.

## Windows x64 MSI

`packaging/windows/MagicPointer.wxs` is a WiX v4 installer definition. GitHub
Actions installs the WiX command-line tool in the runner, builds the release
binary, and produces `MagicPointer-x86_64.msi`. The MSI installs:

* `magic-pointer.exe`;
* the CMD and PowerShell launchers;
* `config.example.json`;
* the README.

The MSI does not store API keys, create a service, require administrator
privileges at first run, or modify the user configuration. The first launch
creates the configuration and asks for the API key/provider/model.

The MSI also creates a **Magic Pointer** Start Menu shortcut. The shortcut
launches the normal desktop window without a console. The first-run wizard is
graphical and asks for the API key, provider, and model.

The current executable is a desktop provider/configuration host. The shared
capture path is implemented, while the following native adapters remain
platform-specific follow-up work:

* Windows: requested execution level, Graphics Capture/UI Automation usage,
  and a tray/overlay host.
* macOS: Screen Recording and Accessibility usage descriptions in
  `Info.plist`, plus a hardened runtime entitlement for the signed app.

The current release is `v0.2.0`
([release page](https://github.com/prluigileoni-png/Magic-pointer-cross-platform/releases/tag/v0.2.0)).
Future `v*` tags use the same CI pipeline and publish the Windows MSI,
Windows portable archives, and macOS ARM64 packages.

The MSI keeps the stable UpgradeCode in `MagicPointer.wxs` and receives the
semantic version from the release tag. `MajorUpgrade` blocks downgrades and
replaces the previous installation in the same folder, including the Start
Menu shortcut, while per-user configuration remains outside the install
directory.

## Windows ARM64

CI produces a native `aarch64-pc-windows-msvc` portable archive. The x64 MSI
can also run on ARM64 Windows through Microsoft's x64 emulation, but the ARM64
archive is the preferred native package. CI also produces
`MagicPointer-aarch64.msi`, which installs the native ARM64 GUI and Start Menu
shortcut using the same upgrade family and release-tag versioning.

## macOS ARM64

CI produces both a portable tarball and a `Magic Pointer.app` ZIP bundle for
Apple Silicon, plus a reproducible DMG. The bundle includes the screen-capture
usage description. It is unsigned unless Apple signing credentials are
configured in CI, so Gatekeeper may require explicit approval on first launch.
