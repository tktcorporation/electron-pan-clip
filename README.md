# clip-filepaths

📋 Copy and read file paths and text from clipboard - A cross platform utility.

<img height="100" src="https://github.com/user-attachments/assets/836b665b-5a53-4b22-b8dc-4cc77a106999" >

## Overview

This library provides functionality for copying and reading file paths and text from the clipboard in Electron applications. It supports Windows, macOS, and Linux platforms. 

Built with [napi-rs](https://napi.rs/) and Rust, this library offers:
- **Multiple file path support**: Copy and read multiple file paths at once
- **Text support**: Read text from the clipboard
- Full TypeScript support
- Cross-platform compatibility

## Motivation

When developing Electron applications, I needed a way to copy multiple photo file paths to the clipboard at once and read content from the clipboard. This library was created to provide a simple solution for batch file path operations, making photo management and other tasks more efficient.

## Installation

```bash
npm install clip-filepaths
```

## Usage

### Copying File Paths

```typescript
import { writeClipboardFilePaths } from 'clip-filepaths';

// Copy file paths to clipboard
const filePaths: string[] = ['/path/to/file1.txt', '/path/to/file2.jpg'];
writeClipboardFilePaths(filePaths);
console.log('File paths copied to clipboard successfully');
```

### Reading Clipboard Content

```typescript
import { readClipboardFilePaths } from 'clip-filepaths';

// Read both file paths and text from clipboard
const content = readClipboardFilePaths();
console.log('File paths:', content.filePaths);
console.log('Text:', content.text);

// Check if clipboard has file paths
if (content.filePaths.length > 0) {
  console.log('Clipboard contains file paths');
}

// Check if clipboard has text
if (content.text) {
  console.log('Clipboard contains text');
}
```

### Clear Clipboard

```typescript
import { writeClipboardFilePaths } from 'clip-filepaths';

// Clear clipboard content by passing an empty array
writeClipboardFilePaths([]);
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
