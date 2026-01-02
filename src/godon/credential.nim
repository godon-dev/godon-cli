import std/[httpclient, json, strutils, uri, tables]
import yaml
import client, types

# Credential API client methods

proc listCredentials*(client: GodonClient): ApiResponseList[Credential] =
  ## List all credentials
  let url = client.baseUrl() & "/credentials"
  
  try:
    let response = client.httpClient.get(url)
    if response.code == Http200:
      let body = parseJson(response.body)
      let credentials = body["credentials"]
      
      var result: seq[Credential] = @[]
      for cred in credentials.items:
        result.add(parseCredentialFromJson(cred))
      
      return ApiResponseList[Credential](
        success: true,
        data: result,
        error: ""
      )
    else:
      return ApiResponseList[Credential](
        success: false,
        data: @[],
        error: "Failed to list credentials: " & response.status
      )
  except Exception as e:
    return ApiResponseList[Credential](
      success: false,
      data: @[],
      error: "Exception: " & e.msg
    )

proc createCredential*(client: GodonClient, credentialData: JsonNode): ApiResponseSingle[Credential] =
  ## Create a new credential
  let url = client.baseUrl() & "/credentials"
  
  try:
    client.httpClient.headers = newHttpHeaders({"Content-Type": "application/json"})
    let response = client.httpClient.post(url, $credentialData)
    if response.code == Http201:
      let body = parseJson(response.body)
      let credential = parseCredentialFromJson(body["credential"])
      
      return ApiResponseSingle[Credential](
        success: true,
        data: credential,
        error: ""
      )
    else:
      return ApiResponseSingle[Credential](
        success: false,
        data: Credential(),
        error: "Failed to create credential: " & response.status
      )
  except Exception as e:
    return ApiResponseSingle[Credential](
      success: false,
      data: Credential(),
      error: "Exception: " & e.msg
    )

proc getCredential*(client: GodonClient, credentialId: string): ApiResponseSingle[Credential] =
  ## Get a specific credential by ID (including content)
  let url = client.baseUrl() & "/credentials/" & encodeUrl(credentialId)
  
  try:
    let response = client.httpClient.get(url)
    if response.code == Http200:
      let body = parseJson(response.body)
      let credential = parseCredentialFromJson(body["credential"])
      
      return ApiResponseSingle[Credential](
        success: true,
        data: credential,
        error: ""
      )
    else:
      return ApiResponseSingle[Credential](
        success: false,
        data: Credential(),
        error: "Failed to get credential: " & response.status
      )
  except Exception as e:
    return ApiResponseSingle[Credential](
      success: false,
      data: Credential(),
      error: "Exception: " & e.msg
    )

proc deleteCredential*(client: GodonClient, credentialId: string): ApiResponse[JsonNode] =
  ## Delete a credential by ID
  let url = client.baseUrl() & "/credentials/" & encodeUrl(credentialId)
  
  try:
    let response = client.httpClient.delete(url)
    if response.code == Http200:
      let body = parseJson(response.body)
      
      return ApiResponse[JsonNode](
        success: true,
        data: body,
        error: ""
      )
    else:
      return ApiResponse[JsonNode](
        success: false,
        data: nil,
        error: "Failed to delete credential: " & response.status
      )
  except Exception as e:
    return ApiResponse[JsonNode](
      success: false,
      data: nil,
      error: "Exception: " & e.msg
    )

proc createCredentialFromYaml*(client: GodonClient, yamlContent: string): ApiResponseSingle[Credential] =
  ## Create credential from YAML content
  ## This is a convenience method that parses YAML and converts to JSON
  
  try:
    # Parse YAML to a generic table, then construct JsonNode manually
    let yamlData = yaml.loadAs[Table[string, string]](yamlContent)
    
    # Build JsonNode from the parsed YAML
    var credentialData = newJObject()
    
    # Add required fields
    credentialData.add("name", newJString(yamlData["name"]))
    credentialData.add("credential_type", newJString(yamlData["credential_type"]))
    
    # Add optional fields
    if "description" in yamlData:
      credentialData.add("description", newJString(yamlData["description"]))
    else:
      credentialData.add("description", newJString(""))
    
    # Ensure content field exists for credentials
    if "content" in yamlData:
      credentialData.add("content", newJString(yamlData["content"]))
    else:
      return ApiResponseSingle[Credential](
        success: false,
        data: Credential(),
        error: "Missing required field: content"
      )
    
    return client.createCredential(credentialData)
  except Exception as e:
    return ApiResponseSingle[Credential](
      success: false,
      data: Credential(),
      error: "Failed to parse YAML: " & e.msg
    )