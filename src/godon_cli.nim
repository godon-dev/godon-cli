import std/[parseopt, strutils, os, json]
import godon/[client, breeder, types]

proc writeHelp() =
  echo """Godon CLI - Command line interface for Godon API

Usage:
  godon_cli [options] <command> [command-options]

Commands:
  breeder list                           List all configured breeders
  breeder create --file <path>           Create a breeder from file
  breeder show --id <id>             Show breeder details
  breeder update --file <path>           Update a breeder from file
  breeder purge --id <id>            Delete a breeder

Global Options:
  --hostname, -h <host>     Godon hostname (default: localhost)
  --port, -p <port>         Godon port (default: 8080)
  --api-version, -v <ver>   API version (default: v0)
  --help, -h                Show this help message

Examples:
  godon_cli breeder list
  godon_cli --hostname api.example.com --port 9090 breeder list
  godon_cli breeder create --file breeder.yaml
  godon_cli breeder show --id 550e8400-e29b-41d4-a716-446655440000
"""

proc writeError(message: string) =
  stderr.writeLine("Error: " & message)
  quit(1)

proc parseArgs(): (string, string, int, string, seq[string]) =
  var command = ""
  var hostname = "localhost"
  var port = 8080
  var apiVersion = "v0"
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
      of "id":
        # Reconstruct as argument for subcommand parsing  
        args.add("--id=" & val)
      of "help", "h":
        writeHelp()
        quit(0)
      else:
        writeError("Unknown option: " & key)
    
    of cmdEnd:
      discard

  if command.len == 0:
    writeHelp()
    quit(0)

  (command, hostname, port, apiVersion, args)

proc handleBreederCommand(client: GodonClient, command: string, args: seq[string]) =
  let subCommand = if args.len > 0: args[0] else: ""
  
  case subCommand:
  of "list":
    echo "Listing breeders..."
    let response = client.listBreeders()
    if response.success:
      echo "Breeders:"
      for breeder in response.data:
        echo "  ID: ", breeder.id
        echo "  Name: ", breeder.name
        echo "  Status: ", breeder.status
        echo "  Created: ", breeder.createdAt
        echo "  ---"
    else:
      writeError(response.error)
  
  of "create":
    var file = ""
    for arg in args:
      if arg.startsWith("--file="):
        file = arg.split("=")[1]
        break
    
    if file.len == 0:
      writeError("breeder create requires --file <path>")
    
    if not fileExists(file):
      writeError("File not found: " & file)
    
    echo "Creating breeder from file: ", file
    let content = readFile(file)
    let response = client.createBreederFromYaml(content)
    if response.success:
      echo "Breeder created successfully:"
      echo "  ID: ", response.data.id
      echo "  Name: ", response.data.name
      echo "  Status: ", response.data.status
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
    
    echo "Getting breeder details for ID: ", id
    let response = client.getBreeder(id)
    if response.success:
      echo "Breeder Details:"
      echo "  ID: ", response.data.id
      echo "  Name: ", response.data.name
      echo "  Status: ", response.data.status
      echo "  Config: ", pretty(response.data.config)
      echo "  Created: ", response.data.createdAt
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
    
    echo "Updating breeder from file: ", file
    let content = readFile(file)
    let response = client.updateBreederFromYaml(content)
    if response.success:
      echo "Breeder updated successfully:"
      echo "  ID: ", response.data.id
      echo "  Name: ", response.data.name
      echo "  Status: ", response.data.status
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
    
    echo "Deleting breeder with ID: ", id
    let response = client.deleteBreeder(id)
    if response.success:
      echo "Breeder deleted successfully"
      if response.data != nil:
        echo "Response: ", pretty(response.data)
    else:
      writeError(response.error)
  
  else:
    writeError("Unknown breeder command: " & subCommand)

let (command, hostname, port, apiVersion, args) = parseArgs()

let godonClient = newGodonClient(hostname, port, apiVersion)

case command:
of "breeder":
  if args.len == 0:
    writeError("breeder command requires a subcommand (list, create, show, update, purge)")
  handleBreederCommand(godonClient, command, args)

else:
  writeError("Unknown command: " & command)