# Magic Pointer (native Rust desktop app)

Magic Pointer is a modern Rust desktop application inspired by the original
Magic Pointer Windhawk mod. It provides a clean graphical window, screen
capture, provider configuration, and a first-run setup wizard for Gemini,
OpenAI, Anthropic, and NVIDIA-compatible APIs.

## Quick start

### Windows x64 (recommended)

Download and run [MagicPointer-x86_64.msi from the latest
release](https://github.com/prluigileoni-png/Magic-pointer-cross-platform/releases/latest).
After installation, open **Start Menu -> Magic Pointer**. The app starts without
a console and shows the graphical setup wizard. Enter the API key, choose the
provider and model, then select **Save and continue**. Settings are stored in
your per-user configuration directory; API keys are not embedded in the
installer.

The main window includes:

* a modern light/dark interface;
* a question editor and clear loading/error states;
* **Ask** and **Capture screen and ask** actions;
* a sidebar with the main workspace and **Settings**;
* editable provider, model, API key, timeout, language, and capture options.

To repeat the wizard, use the app's Settings page or run:

```powershell
.\magic-pointer.exe --setup
```

The portable x64 archive includes the same GUI, plus CMD and PowerShell
launchers. A tagged `v*` build publishes the MSI and portable packages as a
GitHub Release.

### Windows ARM64

Use the native `magic-pointer-aarch64-pc-windows-msvc.tar.gz` package from the
release assets. The x64 MSI can run through Windows ARM64 x64 emulation, but
the ARM64 archive is preferred for native execution. There is intentionally no
ARM64 MSI in the current packaging pipeline.

### macOS ARM64

Download `magic-pointer-aarch64-apple-darwin.app.zip` for Apple Silicon, or the
matching portable tarball. Unzip the app to Applications and open **Magic
Pointer**. The CI app bundle is unsigned unless Apple signing credentials are
configured, so macOS may require an explicit Gatekeeper approval. Grant Screen
Recording permission when using screen capture.

For development, use:

```powershell
cargo run --release
```

For development, use `cargo run --release`. The first launch opens the same
GUI wizard. The portable core can capture the primary display and send a PNG
to a vision-capable provider. Global hotkey activation, OCR, the animated
overlay, browser/clipboard integration, and full UI Automation parity still
require native adapters and permissions.

The Windows launcher is intentionally a thin script beside the executable: it
does not install services, modify the registry, or require administrator
privileges.

## Targets

* Windows x64: `x86_64-pc-windows-msvc`
* Windows ARM64: `aarch64-pc-windows-msvc`
* macOS ARM64: `aarch64-apple-darwin`

See [docs/PORTING.md](docs/PORTING.md) for architecture, packaging guidance,
permission requirements, and the exact technical limitation caused by the
original module being injected by Windhawk into a Windows process.

The original 14k-line Windhawk file is the behavioral specification; this
project deliberately does not inject into processes or pretend that a normal
cross-platform app can reproduce Windhawk's loader semantics.
