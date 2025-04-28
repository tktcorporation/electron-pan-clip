# Release Workflow

This document describes the release process for electron-pan-clip.

## NPM Token Configuration

1. Generate a new access token in your npm account:
   - Log in to [npm website](https://www.npmjs.com/)
   - Click profile icon → Access Tokens → Generate New Token
   - Copy the token

2. Add `NPM_TOKEN` secret to GitHub repository:
   - Go to repository "Settings" → "Secrets and variables" → "Actions"
   - Click "New repository secret"
   - Name: `NPM_TOKEN`, Value: copied npm token

## Release Workflows

This repository has two types of release workflows:

### 1. Auto Release

Detects version changes in `package.json` and automatically starts the release process:

1. **Version Change Detection**: Detects version changes in `package.json` on main branch
2. **CHANGELOG Generation**: Automatically generates CHANGELOG using git-cliff
3. **Release Creation**: Creates GitHub release and tags
4. **npm Publishing**: Publishes package to npm registry
5. **Smoke Test**: Tests installation and basic functionality after publishing

To use this workflow, simply update the version in `package.json` and push to main branch. The process will run automatically.

### 2. Manual Release (Version Bump)

Workflow for manual version updates and releases:

1. Open "Actions" tab in GitHub repository
2. Select "Version Bump" workflow
3. Click "Run workflow"
4. Select update type (patch, minor, major) and run

This will execute:
- Package version update
- Automatic CHANGELOG generation with git-cliff
- Commit changes and create tag
- Push new tag 