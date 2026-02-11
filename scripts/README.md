# Scripts

Utility scripts for foxd development and release management.

## bump-version.sh

Automates version bumping and release tagging.

### Usage

```bash
./scripts/bump-version.sh <major|minor|patch>
```

### What it does

1. **Reads current version** from `daemon/Cargo.toml`
2. **Bumps the version** according to semantic versioning:
   - `major`: 1.0.0 → 2.0.0
   - `minor`: 1.0.0 → 1.1.0
   - `patch`: 1.0.0 → 1.0.1
3. **Updates version files**:
   - `daemon/Cargo.toml`
   - `console/package.json`
4. **Shows binary information** (name, version, platforms, etc.)
5. **Creates git tag** in the format `v<version>`
6. **Shows push command** to trigger the release workflow

### Examples

Bump patch version (bug fixes):

```bash
./scripts/bump-version.sh patch
```

Bump minor version (new features):

```bash
./scripts/bump-version.sh minor
```

Bump major version (breaking changes):

```bash
./scripts/bump-version.sh major
```

### Release Process

1. Make your changes and commit them
2. Run the bump script:
   ```bash
   ./scripts/bump-version.sh patch
   ```
3. Review the changes and confirm
4. The script will:
   - Update version files
   - Create a commit
   - Create a git tag
   - Show you the push command
5. Push to trigger the release:
   ```bash
   git push origin main && git push origin v<version>
   ```
6. GitHub Actions will build binaries for all platforms and create a release

### Supported Platforms

The release workflow builds for:

- **Linux**: x86_64, ARM64, ARMv7
- **Windows**: x86_64, ARM64
