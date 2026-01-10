## Breeder API Methods
## Implementation of breeder-related API endpoints

import std/[httpclient, json, strutils, uri]
import yaml, yaml/tojson
import client, types

proc listBreeders*(client: GodonClient): ApiResponse[seq[BreederSummary]] =
  ## List all configured breeders
  try:
    let url = client.baseUrl() & "/breeders"
    let response = client.httpClient.get(url)
    result = handleResponse[seq[BreederSummary]](client, response)
  except CatchableError as e:
    result = ApiResponse[seq[BreederSummary]](success: false, data: @[], error: e.msg)

proc createBreeder*(client: GodonClient, request: BreederCreateRequest): ApiResponse[BreederSummary] =
  ## Create a new breeder
  try:
    let url = client.baseUrl() & "/breeders"
    # Config is already a JsonNode, no parsing needed
    var jsonData = %*{
      "name": request.name,
      "config": request.config
    }

    # Debug: Show what we're about to send
    when defined(debug):
      echo "DEBUG: Sending JSON to ", url
      echo "DEBUG: JSON string length: ", ($jsonData).len
      echo "DEBUG: Full JSON: ", $jsonData

    echo "Sending JSON: ", $jsonData
    client.httpClient.headers = newHttpHeaders({"Content-Type": "application/json"})
    let response = client.httpClient.post(url, $jsonData)
    result = handleResponse[BreederSummary](client, response)
  except CatchableError as e:
    result = ApiResponse[BreederSummary](success: false, data: default(BreederSummary), error: e.msg)

proc createBreederFromYamlWithName*(client: GodonClient, yamlContent: string, name: string): ApiResponse[BreederSummary] =
  ## Create a breeder from YAML content with explicit name parameter
  ## YAML contains only the config (no name field), name comes from parameter
  try:
    # Parse YAML and convert to JsonNode
    let jsonNodes = loadToJson(yamlContent)

    # Take the first document (should be only one)
    if jsonNodes.len == 0:
      return ApiResponse[BreederSummary](success: false, data: default(BreederSummary), error: "No YAML documents found")

    let configNode = jsonNodes[0]

    # Debug: Show parsed structure
    when defined(debug):
      echo "DEBUG: Parsed YAML to JsonNode"
      echo "DEBUG: ConfigNode keys: ", configNode.keys
      echo "DEBUG: ConfigNode kind: ", configNode.kind
      echo "DEBUG: YAML content length: ", yamlContent.len
      echo "DEBUG: JsonNode size: ", $$configNode.len

    # Create BreederCreateRequest with name from parameter and config from YAML
    let request = BreederCreateRequest(
      name: name,
      config: configNode
    )
    result = client.createBreeder(request)
  except CatchableError as e:
    result = ApiResponse[BreederSummary](success: false, data: default(BreederSummary), error: e.msg)

proc getBreeder*(client: GodonClient, uuid: string): ApiResponse[Breeder] =
  ## Get breeder details by UUID
  try:
    let url = client.baseUrl() & "/breeders/" & encodeUrl(uuid)
    let response = client.httpClient.get(url)
    result = handleResponse[Breeder](client, response)
  except CatchableError as e:
    result = ApiResponse[Breeder](success: false, data: default(Breeder), error: e.msg)

proc updateBreeder*(client: GodonClient, request: BreederUpdateRequest): ApiResponse[Breeder] =
  ## Update an existing breeder
  try:
    let url = client.baseUrl() & "/breeders/" & encodeUrl(request.uuid)
    # Config is a string (JSON), parse it first
    var jsonData = %*{
      "name": request.name,
      "description": request.description,
      "config": parseJson(request.config)
    }
    client.httpClient.headers = newHttpHeaders({"Content-Type": "application/json"})
    let response = client.httpClient.put(url, $jsonData)
    result = handleResponse[Breeder](client, response)
  except CatchableError as e:
    result = ApiResponse[Breeder](success: false, data: default(Breeder), error: e.msg)

proc updateBreederFromYaml*(client: GodonClient, yamlContent: string): ApiResponse[Breeder] =
  ## Update a breeder from YAML content
  try:
    # Parse YAML and convert to JsonNode
    let jsonNodes = loadToJson(yamlContent)

    # Take the first document (should be only one)
    if jsonNodes.len == 0:
      return ApiResponse[Breeder](success: false, data: default(Breeder), error: "No YAML documents found")

    let yamlData = jsonNodes[0]

    # Extract fields from YAML
    let uuid = if yamlData.hasKey("uuid"): yamlData["uuid"].getStr() else: ""
    let name = if yamlData.hasKey("name"): yamlData["name"].getStr() else: ""
    let description = if yamlData.hasKey("description"): yamlData["description"].getStr() else: ""

    # Config should be a nested object - convert to JSON string
    let config = if yamlData.hasKey("config"): $yamlData["config"] else: "{}"

    let request = BreederUpdateRequest(
      uuid: uuid,
      name: name,
      description: description,
      config: config
    )
    result = client.updateBreeder(request)
  except CatchableError as e:
    result = ApiResponse[Breeder](success: false, data: default(Breeder), error: e.msg)

proc deleteBreeder*(client: GodonClient, uuid: string): ApiResponse[JsonNode] =
  ## Delete/purge a breeder by UUID
  try:
    let url = client.baseUrl() & "/breeders/" & encodeUrl(uuid)
    let response = client.httpClient.delete(url)
    result = handleResponse[JsonNode](client, response)
  except CatchableError as e:
    result = ApiResponse[JsonNode](success: false, data: nil, error: e.msg)