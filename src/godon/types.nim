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