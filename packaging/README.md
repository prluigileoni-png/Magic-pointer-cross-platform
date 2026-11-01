# Packaging

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

The current executable is a provider/configuration host. Before shipping a
desktop release, implement the native adapters in `src/platform.rs` and add
the corresponding permission declarations:

* Windows: requested execution level, Graphics Capture/UI Automation usage,
  and a tray/overlay host.
* macOS: Screen Recording and Accessibility usage descriptions in
  `Info.plist`, plus a hardened runtime entitlement for the signed app.
