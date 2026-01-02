## Godon API Types
## Generated based on OpenAPI specification

import std/json

type
  BreederSummary* = object
    id*: string
    name*: string
    status*: string
    createdAt*: string

  Breeder* = object
    id*: string
    name*: string
    status*: string
    config*: JsonNode
    createdAt*: string

  BreederCreateRequest* = object
    name*: string
    config*: string

  BreederUpdateRequest* = object
    uuid*: string
    name*: string
    description*: string
    config*: string

  Credential* = object
    id*: string
    name*: string
    credentialType*: string
    description*: string
    windmillVariable*: string
    createdAt*: string
    lastUsedAt*: string
    content*: string

  ApiConfig* = object
    hostname*: string
    port*: int
    apiVersion*: string

  ApiResponse*[T] = object
    success*: bool
    data*: T
    error*: string

  ApiResponseSingle*[T] = object
    success*: bool
    data*: T
    error*: string

  ApiResponseList*[T] = object
    success*: bool
    data*: seq[T]
    error*: string

  ApiError* = object
    code*: int
    message*: string
    details*: JsonNode

# Helper proc to parse credential from JSON
proc parseCredentialFromJson*(json: JsonNode): Credential =
  result = Credential()
  if json.hasKey("id"):
    result.id = json["id"].getStr()
  if json.hasKey("name"):
    result.name = json["name"].getStr()
  if json.hasKey("credentialType"):
    result.credentialType = json["credentialType"].getStr()
  if json.hasKey("description"):
    result.description = json["description"].getStr()
  else:
    result.description = ""
  if json.hasKey("windmillVariable"):
    result.windmillVariable = json["windmillVariable"].getStr()
  if json.hasKey("createdAt"):
    result.createdAt = json["createdAt"].getStr()
  else:
    result.createdAt = ""
  if json.hasKey("lastUsedAt"):
    result.lastUsedAt = json["lastUsedAt"].getStr()
  else:
    result.lastUsedAt = ""
  if json.hasKey("content"):
    result.content = json["content"].getStr()
  else:
    result.content = ""