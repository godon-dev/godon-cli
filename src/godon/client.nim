## Godon HTTP Client
## Core HTTP client for Godon API

import std/[httpclient, json, uri, strutils]
import types

const
  DefaultHostname* = "localhost"
  DefaultPort* = 8080
  DefaultApiVersion* = "v0"

type
  GodonClient* = ref object
    config*: ApiConfig
    httpClient*: HttpClient

proc newGodonClient*(hostname: string = DefaultHostname, 
                     port: int = DefaultPort, 
                     apiVersion: string = DefaultApiVersion): GodonClient =
  ## Create a new Godon API client
  let config = ApiConfig(
    hostname: hostname,
    port: port,
    apiVersion: apiVersion
  )
  
  let httpClient = newHttpClient()
  GodonClient(config: config, httpClient: httpClient)

proc baseUrl*(client: GodonClient): string =
  ## Get the base URL for API requests
  result = "http://" & client.config.hostname & ":" & $client.config.port

proc handleResponse*[T](client: GodonClient; response: Response): ApiResponse[T] =
  ## Handle HTTP response and convert to ApiResponse
  let statusCode = parseInt(split(response.status, " ")[0])
  if statusCode >= 200 and statusCode < 300:
    try:
      echo "Raw response body: ", response.body
      let jsonData = parseJson(response.body)
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