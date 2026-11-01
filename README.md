# Magic Pointer (native Rust)

This repository is a native Rust port foundation for the Magic Pointer
Windhawk mod. It supports the Gemini, OpenAI, Anthropic and NVIDIA provider
families through one portable executable and persists settings as JSON.

## Quick start

```powershell
cargo run --release
```

Set `api_key`, `provider`, and `model` in the generated configuration file (or
copy `config.example.json`). The prompt loop is usable immediately. Use
`:capture <question>` to capture the primary display and send it to the
selected vision-capable provider. Global hotkey activation, OCR, and the
animated overlay still require native Windows/macOS adapters and permissions.

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
