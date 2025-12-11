# Package information

version       = "0.1.0"
author        = "Matthias Tafelmeier"
description   = "CLI for the Godon API"
license       = "AGPL-3.0"

# Dependencies

requires "nim >= 2.0.0"

# Task definitions

task build, "Build the CLI":
  exec "nim c -d:release -o:bin/godon_cli src/godon_cli.nim"

task build_debug, "Build the CLI with debug symbols":
  exec "nim c -g -o:bin/godon_cli src/godon_cli.nim"

task clean, "Clean build artifacts":
  exec "rm -rf bin"

task test, "Run tests":
  exec "nim c -r tests/test_all.nim"

task docker_build, "Build Docker image":
  exec "docker build -t godon-cli:latest ."

task docker_run, "Run Docker image":
  exec "docker run --rm -it godon-cli:latest --help"

# Binary definition
bin = @["godon_cli"]

# Install script (when installed via nimble)
installDirs = @["bin"]
installFiles = @["bin/godon_cli"]
installExt  = @[]