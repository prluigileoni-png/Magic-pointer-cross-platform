# Native Rust port

## What was ported

The original `mod.wh.cpp` is a Windhawk module loaded into `windhawk.exe`. Its
behavior is split into:

1. pointer shake/hotkey activation and state machine;
2. a transparent, animated Win32/GDI overlay;
3. screen selection, screenshot encoding, Windows UI Automation and Windows OCR;
4. browser/clipboard integration;
5. Gemini, OpenAI, Anthropic and NVIDIA HTTP clients;
6. localized settings and provider/model selection.

This repository now contains the portable Rust core for configuration,
provider requests, model selection, timeouts, screen capture as PNG, and the
same four provider families. The executable is intentionally a normal process
rather than an injected module. `:capture <question>` sends the captured PNG
to the configured provider.

## Build targets

```text
x86_64-pc-windows-msvc   Windows x64
aarch64-pc-windows-msvc  Windows ARM64
aarch64-apple-darwin     macOS Apple Silicon
```

Install Rust targets with `rustup target add ...`, then run `cargo build
--release --target <target>`. CI/package definitions should produce one
installer per target; binaries cannot be embedded in source control as a
substitute for a reproducible build.

## Important compatibility limit

Windhawk hooks are process injection and are specific to the Windows process
hosting the mod. A normal Rust application cannot reproduce that mechanism on
macOS, and it should not inject into arbitrary processes. The native adapters
must instead use supported OS APIs:

* Windows: a desktop overlay, `RegisterHotKey`/low-level input, Windows
  Graphics Capture, UI Automation and Windows OCR, with explicit permissions
  where required.
* macOS: an accessory app, global event monitor, Screen Recording and
  Accessibility permissions, CGWindow/ScreenCaptureKit, Vision OCR and a
  borderless overlay window.

The current portable binary implements display capture through the
cross-platform `screenshots` crate. Global hotkey activation, OCR, and the
animated overlay remain separate native adapters. This is deliberate:
silently claiming full Windhawk equivalence would lose security boundaries and
produce a misleading build.

## Configuration and secrets

Configuration is stored in the platform configuration directory as JSON.
Provider keys are never committed. For production installers, replace the
JSON key field with the OS credential store (Windows Credential Manager and
macOS Keychain).

## First launch

The executable creates a default configuration on first start. If no API key
is present, it runs a setup wizard asking for the key, provider, and model, then
saves the result. The wizard can be explicitly rerun with `--setup`. The
Windows x64 package includes `start-magic-pointer.cmd` and
`Start-MagicPointer.ps1`; both launch the executable without elevation.

## Original source

The attached Windhawk source remains the behavioral reference. It contains
Windows-only COM, WinHTTP, GDI, UI Automation, OCR and shell integrations that
must be reimplemented behind `cfg(target_os)` adapters rather than copied
directly into shared Rust code.
