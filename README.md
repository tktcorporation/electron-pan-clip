# clip-filepaths

A cross-platform file path clipboard utility for Electron applications that supports copying multiple file paths at once.

<img height="100" src="https://github.com/user-attachments/assets/836b665b-5a53-4b22-b8dc-4cc77a106999" >

## Overview

This library provides functionality for copying file paths to the clipboard in Electron applications. It supports Windows, macOS, and Linux platforms. Note that this library copies file paths (references) to the clipboard, not the actual file contents.

Built with [napi-rs](https://napi.rs/) and Rust, this library offers:
- **Multiple file path support**: This library allows copying multiple file paths at once
- Full TypeScript support
- Cross-platform compatibility

## Motivation

When developing Electron applications, I needed a way to copy multiple photo file paths to the clipboard at once. Existing packages only supported copying single file paths, which was inefficient for tasks like photo management. This library was created to fill that gap and provide a simple solution for batch file path operations.

## Installation

```bash
npm install clip-filepaths
```

## Usage

```typescript
import { copyFiles } from 'clip-filepaths';

// Copy file paths to clipboard
const filePaths: string[] = ['/path/to/file1.txt', '/path/to/file2.jpg'];
copyFiles(filePaths);
console.log('File paths copied to clipboard successfully');
```

## Demo

A demo application is available for testing the functionality. If you want to see it in action, check out the following repository:

- [copy-file-paths-electron](https://github.com/tktcorporation/copy-file-paths-electron) - Demo Electron application

## Platform Support

- ✅ macOS: Supported and tested
- ✅ Windows: Supported and tested
- ⚠️ Linux: Supported but not yet tested

## Contributing

We welcome contributions! Please read our [Contributing Guidelines](./docs/CONTRIBUTING.md) before submitting pull requests.

## Support

If you encounter any issues or have questions, please:
1. Check the [existing issues](https://github.com/tktcorporation/clip-filepaths/issues)
2. Create a new issue if your problem hasn't been reported

## Related Projects

- [napi-rs](https://napi.rs/) - Node.js native addon framework
- [Electron](https://www.electronjs.org/) - Cross-platform desktop application framework

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## Acknowledgments

- Thanks to all contributors who have helped improve this project
- Special thanks to the napi-rs team for their excellent framework
