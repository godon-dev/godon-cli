## Godon API Types
## Generated based on OpenAPI specification

import std/json

type
  Breeder* = object
    uuid*: string
    name*: string
    description*: string
    config*: JsonNode
    createdAt*: string
    updatedAt*: string

  BreederCreateRequest* = object
    name*: string
    description*: string
    config*: JsonNode

  BreederUpdateRequest* = object
    uuid*: string
    name*: string
    description*: string
    config*: JsonNode

  ApiConfig* = object
    hostname*: string
    port*: int
    apiVersion*: string

  ApiResponse*[T] = object
    success*: bool
    data*: T
    error*: string

  ApiError* = object
    code*: int
    message*: string
    details*: JsonNode