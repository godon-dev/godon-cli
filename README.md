# Godon CLI

A Nim-based CLI tool for controlling and managing the Godon optimizer breeders via the Godon Control API.

## Features

- List, create, show, update, and delete breeder configurations
- YAML-based configuration files
- RESTful API integration
- Cross-platform support (currently Linux x86_64)

## Installation

### Container Image (Recommended)

The godon-cli is distributed as a container image from the [godon-images](https://github.com/godon-dev/godon-images) repository. This provides distro-agnostic execution with all dependencies included.

```bash
# Run directly
docker run ghcr.io/godon-dev/godon-cli:latest --help

# Example: List breeders
docker run ghcr.io/godon-dev/godon-cli:latest breeder list

# Mount working directory for file operations
docker run -v $(pwd):/work -w /work ghcr.io/godon-dev/godon-cli:latest breeder create --file config.yaml
```

### Building from Source

You can build the CLI from source using Nix flakes (requires Linux x86_64):

```bash
# Clone the repository
git clone https://github.com/godon-dev/godon-cli.git
cd godon-cli

# Build using Nix flakes
nix --experimental-features "nix-command flakes" build

# Run the built binary
./result/bin/godon_cli --help
```

## Usage

### Basic Commands

```bash
# Show help
godon_cli --help

# List all breeders
godon_cli breeder list

# Connect to a different API server
godon_cli --hostname api.example.com --port 9090 breeder list
```

### Configuration Options

```bash
# Connect to a different API server
godon_cli --hostname api.example.com --port 9090 breeder list

# Use HTTPS with SSL verification
godon_cli --hostname https://api.example.com --port 443 breeder list

# Use HTTPS but skip SSL verification (for development/testing)
godon_cli --hostname https://localhost:8443 --insecure breeder list
```

### Breeder Management

#### Create a Breeder

Create a YAML configuration file `breeder.yaml`:

```yaml
name: "genetic-optimizer-1"
config: >
  {"setting1": "value1", "setting2": 42, "optimization_target": "performance"}
```

Then create the breeder:

```bash
godon_cli breeder create --file breeder.yaml
```

#### Show Breeder Details

```bash
godon_cli breeder show --id 550e8400-e29b-41d4-a716-446655440000
```

#### Update a Breeder

Create an update configuration file `breeder_update.yaml`:

```yaml
uuid: "550e8400-e29b-41d4-a716-446655440000"
name: "updated-genetic-optimizer"
description: "Updated optimizer configuration"
config: >
  {"setting1": "new_value1", "setting2": 100}
```

Then update:

```bash
godon_cli breeder update --file breeder_update.yaml
```

#### Delete a Breeder

```bash
godon_cli breeder purge --id 550e8400-e29b-41d4-a716-446655440000
```

## Configuration

The CLI connects to the Godon API using these default settings:

- **Hostname**: `localhost`
- **Port**: `8080`
- **API Version**: `v0`

You can override these using command-line flags:

```bash
# Basic configuration
godon_cli --hostname api.example.com --port 9090 --api-version v1 breeder list

# Protocol and SSL options
godon_cli --hostname http://api.example.com --port 80 breeder list     # HTTP (explicit)
godon_cli --hostname https://api.example.com --port 443 breeder list    # HTTPS (secure)
godon_cli --hostname https://localhost --port 8443 --insecure breeder list  # HTTPS (insecure)
```

## API Specification

This CLI is designed to work with the [Godon Control API](https://github.com/godon-dev/godon-images) which follows OpenAPI 3.0 specification.

## License

This project is licensed under the GNU Affero General Public License v3.0. See the [LICENSE](LICENSE) file for details.

## Releasing

Releases are source-only through GitHub Actions:

1. Create a new release on GitHub with a semantic version tag (e.g., `1.0.0`, `1.0.0-alpha.1`)
2. GitHub automatically provides source archives (tar.gz, zip) for the release
3. Container images are built from the [godon-images](https://github.com/godon-dev/godon-images) repository using this release

**Version format**: Must follow [Semantic Versioning](https://semver.org/) (e.g., `1.0.0`, `2.1.3`, `1.0.0-alpha.1`, `1.0.0+build.1`)

**Note**: This repository focuses on source code releases. Binary distribution is handled via container images for distro-agnostic compatibility.

## Changelog

See the [GitHub Releases](https://github.com/godon-dev/godon-cli/releases) page for version history and changes.
