# electron-pan-clip

A cross-platform file path clipboard utility for Electron applications

## Overview

This library provides functionality for copying file paths to the clipboard in Electron applications. It supports Windows, macOS, and Linux platforms. Note that this library copies file paths (references) to the clipboard, not the actual file contents.

Built with [napi-rs](https://napi.rs/) and Rust, this library offers:
- High performance native implementation
- Full TypeScript support
- Cross-platform compatibility
- Memory safety and thread safety

### Platform Support

- ✅ macOS: Supported and tested
- ✅ Windows: Supported and tested
- ⚠️ Linux: Supported but not yet tested

## Installation

```bash
yarn add electron-pan-clip
```

or

```bash
npm install electron-pan-clip
```

## Usage

```typescript
import { copyFiles } from 'electron-pan-clip';

// Copy file paths to clipboard
const filePaths: string[] = ['/path/to/file1.txt', '/path/to/file2.jpg'];
copyFiles(filePaths);
console.log('File paths copied to clipboard successfully');
```

For detailed examples, please refer to the [examples](./examples) directory.

## Development with pnpm

This project supports pnpm. You can start development with the following commands:

```bash
# Install dependencies
pnpm install

# Build debug version
pnpm build:debug

# Build release version
pnpm build

# Run tests
pnpm test

# Generate documentation
pnpm docs
```

## Development

This project uses [napi-rs](https://napi.rs/) to create Node.js native addons in Rust.

For detailed development information, please refer to [docs/README.md](./docs/README.md).

### Setup

```bash
# Install dependencies
yarn install

# Build
yarn build

# Run tests
yarn test
```

## X11 Support in Development Container

Setup instructions for running Linux X11-related tests in the development container:

1. Install required dependencies
   ```bash
   just install-linux-deps
   ```

2. Run tests with Xvfb
   ```bash
   just test-with-xvfb
   ```

Xvfb provides a virtual framebuffer X11 server, allowing you to run X11 applications without a GUI.
This enables clipboard operation testing in headless environments (environments without a physical display).

### Troubleshooting

If you encounter X11-related issues, check the following:

1. Verify X11 dependencies are correctly installed
   ```bash
   dpkg -l | grep x11
   ```

2. Check if Xvfb is running properly
   ```bash
   Xvfb :99 -screen 0 1280x1024x24 &
   export DISPLAY=:99
   xdpyinfo | head  # Display X11 server information
   ```

## License

MIT