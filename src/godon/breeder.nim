## Breeder API Methods
## Implementation of breeder-related API endpoints

import std/[httpclient, json, strutils, uri]
import client, types

proc listBreeders*(client: GodonClient): ApiResponse[seq[Breeder]] =
  ## List all configured breeders
  try:
    let url = client.baseUrl() & "/breeders"
    let response = client.httpClient.get(url)
    result = handleResponse[seq[Breeder]](client, response)
  except CatchableError as e:
    result = ApiResponse[seq[Breeder]](success: false, data: @[], error: e.msg)

proc createBreeder*(client: GodonClient, request: BreederCreateRequest): ApiResponse[Breeder] =
  ## Create a new breeder
  try:
    let url = client.baseUrl() & "/breeders"
    let jsonData = %*request
    let response = client.httpClient.post(url, $jsonData)
    result = handleResponse[Breeder](client, response)
  except CatchableError as e:
    result = ApiResponse[Breeder](success: false, data: default(Breeder), error: e.msg)

proc parseBreederFromYaml*(yamlContent: string): BreederCreateRequest =
  ## Parse breeder configuration from YAML content
  ## Note: This would require a YAML library like yaml.nim
  ## For now, this is a placeholder for the YAML parsing logic
  ## Users can pass JSON directly or we can add proper YAML parsing later
  try:
    let jsonNode = parseJson(yamlContent)
    result = jsonNode.to(BreederCreateRequest)
  except CatchableError as e:
    raise newException(ValueError, "Failed to parse YAML/JSON: " & e.msg)

proc createBreederFromYaml*(client: GodonClient, yamlContent: string): ApiResponse[Breeder] =
  ## Create a breeder from YAML content
  try:
    let request = parseBreederFromYaml(yamlContent)
    result = client.createBreeder(request)
  except CatchableError as e:
    result = ApiResponse[Breeder](success: false, data: default(Breeder), error: e.msg)

proc getBreeder*(client: GodonClient, uuid: string): ApiResponse[Breeder] =
  ## Get breeder details by UUID
  try:
    let url = client.baseUrl() & "/breeder?uuid=" & encodeUrl(uuid)
    let response = client.httpClient.get(url)
    result = handleResponse[Breeder](client, response)
  except CatchableError as e:
    result = ApiResponse[Breeder](success: false, data: default(Breeder), error: e.msg)

proc updateBreeder*(client: GodonClient, request: BreederUpdateRequest): ApiResponse[Breeder] =
  ## Update an existing breeder
  try:
    let url = client.baseUrl() & "/breeders"
    let jsonData = %*request
    let response = client.httpClient.put(url, $jsonData)
    result = handleResponse[Breeder](client, response)
  except CatchableError as e:
    result = ApiResponse[Breeder](success: false, data: default(Breeder), error: e.msg)

proc parseBreederUpdateFromYaml*(yamlContent: string): BreederUpdateRequest =
  ## Parse breeder update configuration from YAML content
  try:
    let jsonNode = parseJson(yamlContent)
    result = jsonNode.to(BreederUpdateRequest)
  except CatchableError as e:
    raise newException(ValueError, "Failed to parse YAML/JSON: " & e.msg)

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
    let url = client.baseUrl() & "/breeder?uuid=" & encodeUrl(uuid)
    let response = client.httpClient.delete(url)
    result = handleResponse[JsonNode](client, response)
  except CatchableError as e:
    result = ApiResponse[JsonNode](success: false, data: nil, error: e.msg)