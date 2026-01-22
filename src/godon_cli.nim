import std/[parseopt, strutils, os, json]
import yaml, yaml/presenter, yaml/dumping
import godon/[client, breeder, credential, types]

type
  OutputFormat* = enum
    Text = "text"
    Json = "json"
    Yaml = "yaml"

proc writeHelp() =
  echo """Godon CLI - Command line interface for Godon API

Usage:
  godon_cli [options] <command> [command-options]

Commands:
  breeder list                           List all configured breeders
  breeder create --name <name> --file <path>  Create a breeder from file
  breeder show --id <id>             Show breeder details
  breeder update --file <path>           Update a breeder from file
  breeder stop --id <id>            Stop breeder workers (graceful shutdown)
  breeder start --id <id>           Start/resume a stopped breeder
  breeder purge [--force] --id <id>     Delete a breeder (use --force to cancel workers immediately)

  credential list                        List all credentials
  credential create --file <path>        Create a credential from file
  credential show --id <id>          Show credential details (including content)
  credential delete --id <id>         Delete a credential

Global Options:
  --hostname, -h <host>     Godon hostname (default: localhost)
  --port, -p <port>         Godon port (default: 8080)
  --api-version, -v <ver>   API version (default: v0)
  --output, -o <format>     Output format: text, json, or yaml (default: text)
  --force                   Force immediate deletion (skip graceful shutdown)
  --insecure                Skip SSL certificate verification (HTTPS only)
  --help                    Show this help message

Examples:
  godon_cli breeder list
  godon_cli --hostname api.example.com --port 9090 breeder list
  godon_cli breeder create --name my-breeder --file breeder-config.yaml
  godon_cli breeder show --id 550e8400-e29b-41d4-a716-446655440000
  godon_cli breeder stop --id 550e8400-e29b-41d4-a716-446655440000
  godon_cli breeder purge --force --id 550e8400-e29b-41d4-a716-446655440000
  godon_cli credential list
  godon_cli credential create --file credential.yaml
  godon_cli credential show --id 550e8400-e29b-41d4-a716-446655440001
"""

proc writeError(message: string) =
  stderr.writeLine("Error: " & message)
  quit(1)

proc parseArgs(): (string, string, int, string, bool, bool, bool, OutputFormat, seq[string]) =
  var command = ""
  var hostname = "localhost"
  var port = 8080
  var apiVersion = "v0"
  var insecure = false
  var debug = false
  var force = false
  var outputFormat = OutputFormat.Text
  var args: seq[string] = @[]

  var p = initOptParser(commandLineParams())

  for kind, key, val in p.getopt():
    case kind
    of cmdArgument:
      if command.len == 0:
        command = key
      else:
        args.add(key)

    of cmdLongOption, cmdShortOption:
      case key.normalize()
      of "hostname":
        hostname = val
      of "port":
        if val.len == 0:
          writeError("Port option requires a value")
        try:
          port = parseInt(val)
        except ValueError:
          writeError("Invalid port number: " & val)
      of "api-version":
        apiVersion = val
      of "file":
        # Reconstruct as argument for subcommand parsing
        args.add("--file=" & val)
      of "name":
        # Reconstruct as argument for subcommand parsing
        args.add("--name=" & val)
      of "id":
        # Reconstruct as argument for subcommand parsing
        args.add("--id=" & val)
      of "force":
        force = true
      of "insecure":
        insecure = true
      of "debug":
        debug = true
      of "output", "o":
        if val.len == 0:
          writeError("Output option requires a value (text, json, or yaml)")
        case val.normalize()
        of "text":
          outputFormat = OutputFormat.Text
        of "json":
          outputFormat = OutputFormat.Json
        of "yaml":
          outputFormat = OutputFormat.Yaml
        else:
          writeError("Unknown output format: " & val & ". Use text, json, or yaml")
      of "help":
        writeHelp()
        quit(0)
      else:
        writeError("Unknown option: " & key)

    of cmdEnd:
      discard

  if command.len == 0:
    writeHelp()
    quit(0)

  # Also check for DEBUG environment variable
  if existsEnv("DEBUG"):
    debug = true

  (command, hostname, port, apiVersion, insecure, debug, force, outputFormat, args)

proc formatOutput*[T](data: T, outputFormat: OutputFormat) =
  ## Format data according to output format preference
  case outputFormat
  of OutputFormat.Text:
    # Text mode is handled by caller with custom formatting
    discard
  of OutputFormat.Json:
    echo pretty(%*data)
  of OutputFormat.Yaml:
    # Convert to YAML by dumping JSON as text and transforming it
    let jsonString = pretty(%*data)
    var dumper = blockOnlyDumper()
    echo dumper.transform(jsonString)

proc handleBreederCommand(client: GodonClient, command: string, args: seq[string], force: bool, outputFormat: OutputFormat) =
  let subCommand = if args.len > 0: args[0] else: ""
  
  case subCommand:
  of "list":
    let response = client.listBreeders()
    if response.success:
      if outputFormat == OutputFormat.Text:
        echo "Breeders:"
        for breeder in response.data:
          echo "  ID: ", breeder.id
          echo "  Name: ", breeder.name
          echo "  Status: ", breeder.status
          echo "  Created: ", breeder.createdAt
          echo "  ---"
      else:
        formatOutput(response.data, outputFormat)
    else:
      writeError(response.error)
  
  of "create":
    var file = ""
    var name = ""
    for arg in args:
      if arg.startsWith("--file="):
        file = arg.split("=")[1]
      elif arg.startsWith("--name="):
        name = arg.split("=")[1]

    if file.len == 0:
      writeError("breeder create requires --file <path>")

    if name.len == 0:
      writeError("breeder create requires --name <name>")

    if not fileExists(file):
      writeError("File not found: " & file)

    let content = readFile(file)
    let response = client.createBreederFromYamlWithName(content, name)
    if response.success:
      if outputFormat == OutputFormat.Text:
        echo "Breeder created successfully:"
        echo "  ID: ", response.data.id
        echo "  Name: ", response.data.name
        echo "  Status: ", response.data.status
      else:
        formatOutput(response.data, outputFormat)
    else:
      writeError(response.error)
  
  of "show":
    var id = ""
    for arg in args:
      if arg.startsWith("--id="):
        id = arg.split("=")[1]
        break
    
    if id.len == 0:
      writeError("breeder show requires --id <id>")

    let response = client.getBreeder(id)
    if response.success:
      if outputFormat == OutputFormat.Text:
        echo "Breeder Details:"
        echo "  ID: ", response.data.id
        echo "  Name: ", response.data.name
        echo "  Status: ", response.data.status
        echo "  Config: ", pretty(response.data.config)
        echo "  Created: ", response.data.createdAt
      else:
        formatOutput(response.data, outputFormat)
    else:
      writeError(response.error)
  
  of "update":
    var file = ""
    for arg in args:
      if arg.startsWith("--file="):
        file = arg.split("=")[1]
        break
    
    if file.len == 0:
      writeError("breeder update requires --file <path>")
    
    if not fileExists(file):
      writeError("File not found: " & file)

    let content = readFile(file)
    let response = client.updateBreederFromYaml(content)
    if response.success:
      if outputFormat == OutputFormat.Text:
        echo "Breeder updated successfully:"
        echo "  ID: ", response.data.id
        echo "  Name: ", response.data.name
        echo "  Status: ", response.data.status
      else:
        formatOutput(response.data, outputFormat)
    else:
      writeError(response.error)
  
  of "purge":
    var id = ""
    for arg in args:
      if arg.startsWith("--id="):
        id = arg.split("=")[1]
        break

    if id.len == 0:
      writeError("breeder purge requires --id <id>")

    let response = client.deleteBreeder(id, force)
    if response.success:
      if outputFormat == OutputFormat.Text:
        if force:
          echo "Breeder force deleted (workers cancelled): ", id
        else:
          echo "Breeder deleted: ", id
      else:
        formatOutput(response.data, outputFormat)
    else:
      writeError(response.error)

  of "stop":
    var id = ""
    for arg in args:
      if arg.startsWith("--id="):
        id = arg.split("=")[1]
        break

    if id.len == 0:
      writeError("breeder stop requires --id <id>")

    let response = client.stopBreeder(id)
    if response.success:
      if outputFormat == OutputFormat.Text:
        echo "Breeder stop requested (graceful shutdown): ", id
        echo "Workers will finish current trial before stopping."
      else:
        formatOutput(response.data, outputFormat)
    else:
      writeError(response.error)

  of "start":
    var id = ""
    for arg in args:
      if arg.startsWith("--id="):
        id = arg.split("=")[1]
        break

    if id.len == 0:
      writeError("breeder start requires --id <id>")

    let response = client.startBreeder(id)
    if response.success:
      if outputFormat == OutputFormat.Text:
        echo "Breeder started/resumed: ", id
      else:
        formatOutput(response.data, outputFormat)
    else:
      writeError(response.error)

  else:
    writeError("Unknown breeder command: " & subCommand)

proc handleCredentialCommand(client: GodonClient, command: string, args: seq[string], outputFormat: OutputFormat) =
  let subCommand = if args.len > 0: args[0] else: ""
  
  case subCommand:
  of "list":
    let response = client.listCredentials()
    if response.success:
      if outputFormat == OutputFormat.Text:
        echo "Credentials:"
        for credential in response.data:
          echo "  ID: ", credential.id
          echo "  Name: ", credential.name
          echo "  Type: ", credential.credentialType
          echo "  Description: ", credential.description
          echo "  windmillVariable: ", credential.windmillVariable
          echo "  Created: ", credential.createdAt
          echo "  ---"
      else:
        formatOutput(response.data, outputFormat)
    else:
      writeError(response.error)
  
  of "create":
    var file = ""
    for arg in args:
      if arg.startsWith("--file="):
        file = arg.split("=")[1]
        break
    
    if file.len == 0:
      writeError("credential create requires --file <path>")
    
    if not fileExists(file):
      writeError("File not found: " & file)

    let content = readFile(file)
    let response = client.createCredentialFromYaml(content)
    if response.success:
      if outputFormat == OutputFormat.Text:
        echo "Credential created successfully:"
        echo "  ID: ", response.data.id
        echo "  Name: ", response.data.name
        echo "  Type: ", response.data.credentialType
        echo "  windmillVariable: ", response.data.windmillVariable
      else:
        formatOutput(response.data, outputFormat)
    else:
      writeError(response.error)
  
  of "show":
    var id = ""
    for arg in args:
      if arg.startsWith("--id="):
        id = arg.split("=")[1]
        break
    
    if id.len == 0:
      writeError("credential show requires --id <id>")

    let response = client.getCredential(id)
    if response.success:
      if outputFormat == OutputFormat.Text:
        echo "Credential Details:"
        echo "  ID: ", response.data.id
        echo "  Name: ", response.data.name
        echo "  Type: ", response.data.credentialType
        echo "  Description: ", response.data.description
        echo "  windmillVariable: ", response.data.windmillVariable
        echo "  Created: ", response.data.createdAt
        echo "  Last Used: ", response.data.lastUsedAt
        echo "  Content:"
        echo "    ", response.data.content
      else:
        formatOutput(response.data, outputFormat)
    else:
      writeError(response.error)
  
  of "delete":
    var id = ""
    for arg in args:
      if arg.startsWith("--id="):
        id = arg.split("=")[1]
        break
    
    if id.len == 0:
      writeError("credential delete requires --id <id>")

    let response = client.deleteCredential(id)
    if response.success:
      if outputFormat == OutputFormat.Text:
        echo "Credential deleted successfully: ", id
      else:
        formatOutput(response.data, outputFormat)
    else:
      writeError(response.error)
  
  else:
    writeError("Unknown credential command: " & subCommand)

let (command, hostname, port, apiVersion, insecure, debug, force, outputFormat, args) = parseArgs()

let godonClient = newGodonClient(hostname, port, apiVersion, insecure, debug)

case command:
of "breeder":
  if args.len == 0:
    writeError("breeder command requires a subcommand (list, create, show, update, stop, start, purge)")
  handleBreederCommand(godonClient, command, args, force, outputFormat)

of "credential":
  if args.len == 0:
    writeError("credential command requires a subcommand (list, create, show, delete)")
  handleCredentialCommand(godonClient, command, args, outputFormat)

else:
  writeError("Unknown command: " & command)