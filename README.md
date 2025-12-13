# Godon CLI

A Nim-based CLI tool for controlling and managing the Godon optimizer breeders via the Godon Control API.

## Features

- List, create, show, update, and delete breeder configurations
- YAML-based configuration files
- RESTful API integration
- Cross-platform support (currently Linux x86_64)

## Installation

### Binary Download

Download the latest release from the [GitHub Releases](https://github.com/godon-dev/godon-cli/releases) page:

```bash
# Download and extract (replace VERSION with actual version)
wget https://github.com/godon-dev/godon-cli/releases/download/VERSION/godon-cli-VERSION-x86_64-linux.tar.gz
tar -xzf godon-cli-VERSION-x86_64-linux.tar.gz

# Make executable and move to PATH
chmod +x godon_cli
sudo mv godon_cli /usr/local/bin/
```

### Docker

Docker images are built from the [godon-images](https://github.com/godon-dev/godon-images) repository and include the CLI binary.

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
godon_cli --hostname api.example.com --port 9090 --api-version v1 breeder list
```

## API Specification

This CLI is designed to work with the [Godon Control API](https://github.com/godon-dev/godon-images) which follows OpenAPI 3.0 specification.

## License

This project is licensed under the GNU Affero General Public License v3.0. See the [LICENSE](LICENSE) file for details.

## Releasing

Releases are automated through GitHub Actions:

1. Create a new release on GitHub with a semantic version tag (e.g., `1.0.0`, `1.0.0-alpha.1`)
2. GitHub Actions will automatically build and upload `godon-cli-VERSION-x86_64-linux.tar.gz`
3. The compressed archive will be available in the release assets

**Version format**: Must follow [Semantic Versioning](https://semver.org/) (e.g., `1.0.0`, `2.1.3`, `1.0.0-alpha.1`, `1.0.0+build.1`)

## Changelog

See the [GitHub Releases](https://github.com/godon-dev/godon-cli/releases) page for version history and changes.
