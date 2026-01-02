## Godon HTTP Client
## Core HTTP client for Godon API

import std/[httpclient, json, uri, strutils, net]
import types

const
  DefaultHostname* = "localhost"
  DefaultPort* = 8080
  DefaultApiVersion* = "v0"

type
  GodonClient* = ref object
    config*: ApiConfig
    httpClient*: HttpClient
    insecure*: bool

proc newGodonClient*(hostname: string = DefaultHostname, 
                     port: int = DefaultPort, 
                     apiVersion: string = DefaultApiVersion,
                     insecure: bool = false): GodonClient =
  ## Create a new Godon API client
  let config = ApiConfig(
    hostname: hostname,
    port: port,
    apiVersion: apiVersion
  )
  
  # Configure HTTP client with SSL verification settings
  var httpClient: HttpClient
  if insecure:
    # Create insecure SSL context that skips certificate verification
    let sslContext = newContext(verifyMode = CVerifyNone)
    httpClient = newHttpClient(sslContext = sslContext)
  else:
    httpClient = newHttpClient()
  
  GodonClient(config: config, httpClient: httpClient, insecure: insecure)

proc baseUrl*(client: GodonClient): string =
  ## Get the base URL for API requests
  ## Auto-detect protocol if hostname includes https:// or http:// prefix
  let scheme = if client.config.hostname.startsWith("https://"):
                  "https"
                elif client.config.hostname.startsWith("http://"):
                  "http"
                else:
                  "http"  # default to HTTP
  
  # Strip protocol prefix from hostname if present
  let cleanHost = if client.config.hostname.startsWith("https://") or 
                     client.config.hostname.startsWith("http://"):
                     client.config.hostname.split("://", 1)[1]
                   else:
                     client.config.hostname
  
  result = scheme & "://" & cleanHost & ":" & $client.config.port

proc handleResponse*[T](client: GodonClient; response: Response): ApiResponse[T] =
  ## Handle HTTP response and convert to ApiResponse
  let statusCode = parseInt(split(response.status, " ")[0])
  if statusCode >= 200 and statusCode < 300:
    try:
      echo "Raw response body: ", response.body
      let jsonData = parseJson(response.body)
      
      # Handle nested response structures like {"breeders": [...]} or {"credentials": [...]}
      if jsonData.kind == JObject:
        # Check for wrapped list responses
        when T is seq:
          for key, value in jsonData.pairs:
            if value.kind == JArray:
              result = ApiResponse[T](success: true, data: value.to(T), error: "")
              return
        # For single objects, try direct conversion first (bare objects)
        else:
          try:
            result = ApiResponse[T](success: true, data: jsonData.to(T), error: "")
            return
          except CatchableError:
            # If direct conversion fails, try finding wrapped object
            for key, value in jsonData.pairs:
              if value.kind == JObject:
                result = ApiResponse[T](success: true, data: value.to(T), error: "")
                return
      
      # Fallback to direct conversion
      result = ApiResponse[T](success: true, data: jsonData.to(T), error: "")
    except CatchableError as e:
      result = ApiResponse[T](success: false, data: default(T), error: "JSON parse error: " & e.msg)
  else:
    try:
      echo "HTTP Error Response Body: ", response.body
      let errorJson = parseJson(response.body)
      let errorMsg = errorJson{"message"}.getStr("HTTP Error: " & $statusCode)
      result = ApiResponse[T](success: false, data: default(T), error: errorMsg)
    except CatchableError:
      result = ApiResponse[T](success: false, data: default(T), error: "HTTP Error: " & $statusCode)

proc handleError*(client: GodonClient, response: Response): ref CatchableError =
  ## Convert HTTP error response to exception
  let statusCode = parseInt(split(response.status, " ")[0])
  var errorMsg = "HTTP Error: " & $statusCode
  try:
    let errorJson = parseJson(response.body)
    errorMsg = errorJson{"message"}.getStr(errorMsg)
  except CatchableError:
    discard
  
  newException(CatchableError, errorMsg)