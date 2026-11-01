# Magic Pointer (native Rust)

This repository is a native Rust port foundation for the Magic Pointer
Windhawk mod. It supports the Gemini, OpenAI, Anthropic and NVIDIA provider
families through one portable executable and persists settings as JSON.

## Quick start

Download the `magic-pointer-x86_64-pc-windows-msvc` artifact from the GitHub
Actions workflow for Windows x64, extract it, and double-click
`start-magic-pointer.cmd`. The first launch asks for the API key, provider, and
model, then stores the completed configuration in the user config directory.
To repeat the wizard, run:

```powershell
.\magic-pointer.exe --setup
```

For a normal Windows installation, download the
`magic-pointer-windows-x64-installer` artifact and run
`MagicPointer-x86_64.msi`. The MSI installs the executable and both launchers
without storing credentials or requiring an administrator service. A tagged
`v*` build publishes the same MSI and portable archives as a GitHub Release.

For development, use:

```powershell
cargo run --release
```

Set `api_key`, `provider`, and `model` in the generated configuration file (or
copy `config.example.json`). The prompt loop is usable immediately. Use
`:capture <question>` to capture the primary display and send it to the
selected vision-capable provider. Global hotkey activation, OCR, and the
animated overlay still require native Windows/macOS adapters and permissions.

The Windows launcher is intentionally a thin script beside the executable: it
does not install services, modify the registry, or require administrator
privileges.

## Targets

* Windows x64: `x86_64-pc-windows-msvc`
* Windows ARM64: `aarch64-pc-windows-msvc`
* macOS ARM64: `aarch64-apple-darwin`

See [docs/PORTING.md](docs/PORTING.md) for the architecture, packaging
guidance, and the exact technical limitation caused by the original module
being injected by Windhawk into a Windows process.

The original 14k-line Windhawk file is the behavioral specification; this
project deliberately does not inject into processes or pretend that a normal
cross-platform app can reproduce Windhawk's loader semantics.
