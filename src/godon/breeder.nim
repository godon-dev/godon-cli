## Breeder API Methods
## Implementation of breeder-related API endpoints

import std/[httpclient, json, strutils, uri]
import yaml
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
    # Convert config string to JsonNode
    var jsonData = %*{
      "name": request.name,
      "config": parseJson(request.config)
    }
    echo "Sending JSON: ", $jsonData
    client.httpClient.headers = newHttpHeaders({"Content-Type": "application/json"})
    let response = client.httpClient.post(url, $jsonData)
    result = handleResponse[BreederSummary](client, response)
  except CatchableError as e:
    result = ApiResponse[BreederSummary](success: false, data: default(BreederSummary), error: e.msg)

proc parseBreederFromYaml*(yamlContent: string): BreederCreateRequest =
  ## Parse breeder configuration from YAML content using yaml library
  try:
    result = yaml.loadAs[BreederCreateRequest](yamlContent)
  except CatchableError as e:
    raise newException(ValueError, "Failed to parse YAML: " & e.msg)

proc createBreederFromYaml*(client: GodonClient, yamlContent: string): ApiResponse[BreederSummary] =
  ## Create a breeder from YAML content
  try:
    let request = parseBreederFromYaml(yamlContent)
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
    # Convert config string to JsonNode
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

proc parseBreederUpdateFromYaml*(yamlContent: string): BreederUpdateRequest =
  ## Parse breeder update configuration from YAML content
  try:
    result = yaml.loadAs[BreederUpdateRequest](yamlContent)
  except CatchableError as e:
    raise newException(ValueError, "Failed to parse YAML: " & e.msg)

proc updateBreederFromYaml*(client: GodonClient, yamlContent: string): ApiResponse[Breeder] =
  ## Update a breeder from YAML content
  try:
    let request = parseBreederUpdateFromYaml(yamlContent)
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