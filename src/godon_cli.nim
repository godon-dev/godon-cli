import std/[parseopt, strutils, os, json]
import godon/[client, breeder, types]

proc writeHelp() =
  echo """Godon CLI - Command line interface for Godon API

Usage:
  godon_cli [options] <command> [command-options]

Commands:
  breeder list                           List all configured breeders
  breeder create --file <path>           Create a breeder from file
  breeder show --uuid <uuid>             Show breeder details
  breeder update --file <path>           Update a breeder from file
  breeder purge --uuid <uuid>            Delete a breeder

Global Options:
  --hostname, -h <host>     Godon hostname (default: localhost)
  --port, -p <port>         Godon port (default: 8080)
  --api-version, -v <ver>   API version (default: v0)
  --help, -h                Show this help message

Examples:
  godon_cli breeder list
  godon_cli --hostname api.example.com --port 9090 breeder list
  godon_cli breeder create --file breeder.yaml
  godon_cli breeder show --uuid 123e4567-e89b-12d3-a456-426614174000
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
        try:
          port = parseInt(val)
        except ValueError:
          writeError("Invalid port number: " & val)
      of "api-version":
        apiVersion = val
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
        echo "  UUID: ", breeder.uuid
        echo "  Name: ", breeder.name
        echo "  Description: ", breeder.description
        echo "  Created: ", breeder.createdAt
        echo "  Updated: ", breeder.updatedAt
        echo "  ---"
    else:
      writeError(response.error)
  
  of "create":
    var file = ""
    for i in 1 ..< args.len.int:
      if args[i-1] == "--file":
        file = args[i]
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
      echo "  UUID: ", response.data.uuid
      echo "  Name: ", response.data.name
      echo "  Description: ", response.data.description
    else:
      writeError(response.error)
  
  of "show":
    var uuid = ""
    for i in 1 ..< args.len.int:
      if args[i-1] == "--uuid":
        uuid = args[i]
        break
    
    if uuid.len == 0:
      writeError("breeder show requires --uuid <uuid>")
    
    echo "Getting breeder details for UUID: ", uuid
    let response = client.getBreeder(uuid)
    if response.success:
      echo "Breeder Details:"
      echo "  UUID: ", response.data.uuid
      echo "  Name: ", response.data.name
      echo "  Description: ", response.data.description
      echo "  Config: ", pretty(response.data.config)
      echo "  Created: ", response.data.createdAt
      echo "  Updated: ", response.data.updatedAt
    else:
      writeError(response.error)
  
  of "update":
    var file = ""
    for i in 1 ..< args.len.int:
      if args[i-1] == "--file":
        file = args[i]
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
      echo "  UUID: ", response.data.uuid
      echo "  Name: ", response.data.name
      echo "  Description: ", response.data.description
    else:
      writeError(response.error)
  
  of "purge":
    var uuid = ""
    for i in 1 ..< args.len.int:
      if args[i-1] == "--uuid":
        uuid = args[i]
        break
    
    if uuid.len == 0:
      writeError("breeder purge requires --uuid <uuid>")
    
    echo "Deleting breeder with UUID: ", uuid
    let response = client.deleteBreeder(uuid)
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