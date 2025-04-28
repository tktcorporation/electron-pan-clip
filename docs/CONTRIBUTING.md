# Contributing Guidelines

Thank you for your interest in contributing to clip-filepaths! This document provides guidelines for contributing to the project.

## Commit Message Format

We use [Conventional Commits](https://www.conventionalcommits.org/) for our commit messages. This helps us maintain a clear and consistent commit history and automatically generate changelogs.

### Format

```
<type>[(optional scope)]: <description>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `doc`: Documentation changes only
- `perf`: Performance improvement
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `style`: Changes that do not affect code meaning (whitespace, formatting, etc.)
- `test`: Adding or modifying tests
- `chore`: Other changes (build process, etc.)

### Examples

```
feat(core): Add file copy functionality
fix(windows): Resolve path resolution issues on Windows platform
doc: Update README
```

### Notes

- Including a scope helps identify the affected component or module (e.g., `feat(core):`, `fix(windows):`)
- For breaking changes, add `!` after the type/scope or include `BREAKING CHANGE:` in the footer

## Development Process

1. Fork the repository
2. Create a new branch for your feature/fix
3. Make your changes
4. Write or update tests as needed
5. Ensure all tests pass
6. Submit a pull request

## Code Style

- Follow the existing code style
- Use meaningful variable and function names
- Add comments for complex logic
- Keep functions focused and small

## Testing

- Write tests for new features
- Ensure all tests pass before submitting a PR
- Update tests when fixing bugs

## Documentation

- Update documentation when adding new features or changing behavior
- Keep comments and documentation up to date
- Use clear and concise language

## Pull Requests

- Provide a clear description of changes
- Reference any related issues
- Ensure CI checks pass
- Request review from maintainers

Thank you for contributing to clip-filepaths! 