# electron-pan-clip

A cross-platform file clipboard utility for Electron applications

## Project Structure

```
electron-pan-clip/
├── src/                # Source Code
├── tests/              # Tests
├── examples/           # Usage Examples
├── docs/               # Documentation
├── .devcontainer/      # Development Container Configuration
├── Cargo.toml          # Rust Project Configuration
├── package.json        # Node.js Project Configuration
├── index.js            # JavaScript API Entry Point
└── index.d.ts          # TypeScript Type Definitions
```

## Development

### Prerequisites

- Node.js (version specified in `.node-version`)
- Rust (latest stable)
- Yarn (latest stable)

### Installing Dependencies

```bash
yarn install
```

### Building

```bash
yarn build
```

### Testing

```bash
yarn test
```

## Platform-Specific Implementation Notes

### Windows Implementation

The Windows implementation uses `CF_HDROP` format and `DROPFILES` structure for copying files to the clipboard. Key features:

- Uses `windows-sys` crate
- Explicitly defines `CF_HDROP` format (value = 15)
- Manually implements `DROPFILES` structure
- Converts file paths to UTF-16 (wide string) with NULL termination
- Properly manages global memory allocation, locking/unlocking

### macOS Implementation

The macOS implementation uses `NSPasteboard` and `NSURL` for copying files to the clipboard. Key features:

- Uses `objc` and `cocoa` crates
- Implements proper memory management with `NSAutoreleasePool`
- Handles both `public.file-url` and `NSFilenamesPboardType` formats
- Provides comprehensive error handling

### Linux Implementation

The Linux implementation uses `text/uri-list` format for copying files to the clipboard. Key features:

- Uses `arboard` crate for clipboard operations
- Converts file paths to `file://` URIs
- Handles both X11 and Wayland environments
- Provides fallback mechanisms for different clipboard implementations

## Development Environment

This project uses a development container for consistent development environment. The configuration is located in `.devcontainer/`.

### Cross-Compilation

Cross-compilation is configured in `Cross.toml`. The project supports building for multiple platforms:

- Windows (x86_64, arm64)
- macOS (x86_64, arm64)
- Linux (x86_64, arm64)

### Testing

- Unit tests are written in Rust and can be run with `cargo test`
- Integration tests are written in TypeScript and can be run with `yarn test`

## References

- [Rust Cross-Compilation Documentation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [napi-rs Documentation](https://napi.rs/docs/introduction/building-for-multiple-platforms)

## License

MIT 