import AsyncStorage from "@react-native-async-storage/async-storage"
import { hashPassword } from "../utils/password"
import { normalizeId, isValidId } from "../utils/idUtils"

// Global auth context reference - will be set by the AuthContextProvider
let globalAuthContext = null

// Function to set the global auth context
export const setGlobalAuthContext = (authContext) => {
  globalAuthContext = authContext
}

// Base URL for the API - Keeping the IP address as requested
const API_URL = "http://192.168.1.141:8000"

// Helper function for API requests with proper error handling
export const apiRequest = async (endpoint, method = "GET", data = null, skipAuthCheck = false) => {
  // Skip auth check for public endpoints (login, register)
  if (!skipAuthCheck && globalAuthContext) {
    // Only check if auth data exists, let the backend validate the token
    if (!globalAuthContext.hasAuthData()) {
      console.log("No authentication data found, redirecting to login")
      globalAuthContext.invalidateAuth()
      throw new Error("No authentication data")
    }
  }

  const token = await AsyncStorage.getItem("userToken")

  const headers = {
    "Content-Type": "application/json",
  }

  if (token) {
    headers["Authorization"] = `Bearer ${token}`
  }

  const config = {
    method,
    headers,
  }

  if (data && (method === "POST" || method === "PUT")) {
    config.body = JSON.stringify(data)
  }

  try {
    console.log(`Making ${method} request to ${API_URL}${endpoint}`)
    if (data) {
      console.log("With data:", JSON.stringify(data))
    }

    const response = await fetch(`${API_URL}${endpoint}`, config)

    // Handle authentication errors (401, 403)
    if (response.status === 401 || response.status === 403) {
      console.log(`Authentication error from server: ${response.status}`)

      if (globalAuthContext && !skipAuthCheck) {
        globalAuthContext.invalidateAuth()
      }

      throw new Error(`Authentication error: ${response.status}`)
    }

    if (!response.ok) {
      const errorText = await response.text()
      throw new Error(errorText || "Error en la petición")
    }

    // Handle content type
    const contentType = response.headers.get("content-type")
    if (contentType && contentType.includes("application/json")) {
      // JSON response
      const responseData = await response.json()
      console.log(`API Response for ${endpoint}:`, JSON.stringify(responseData, null, 2))
      return responseData
    } else {
      // Text response (like JWT token)
      const text = await response.text()
      return { success: true, data: text }
    }
  } catch (error) {
    // Check if this is an authentication error
    if (
      error.message &&
      (error.message.includes("Authentication error") ||
        error.message.includes("401") ||
        error.message.includes("403") ||
        error.message.includes("token"))
    ) {
      console.log("Authentication error detected:", error.message)

      if (globalAuthContext && !skipAuthCheck) {
        globalAuthContext.invalidateAuth()
      }
    }

    console.error("API Error:", error)
    throw error
  }
}

// Autenticación
export const apiLogin = async (email, password) => {
  try {
    console.log("Login attempt for:", email)

    // Hash the password before sending
    const hashedPassword = await hashPassword(password)
    console.log("Password hashed successfully")

    // Using the correct endpoint format for login
    // Skip auth check for login
    const response = await apiRequest(`/public/users/login/${email}/${hashedPassword}`, "POST", null, true)
    console.log("Login response received:", response)
    return response
  } catch (error) {
    console.error("Login error:", error)
    throw error
  }
}

export const apiRegister = async (name, email, password) => {
  try {
    console.log("Registration attempt for:", email)

    // Hash the password before sending
    const hashedPassword = await hashPassword(password)
    console.log("Password hashed successfully")

    // Usar los nombres de campo correctos según la estructura del backend
    // Skip auth check for register
    const response = await apiRequest(
      "/public/users/register",
      "POST",
      {
        name,
        mail: email, // Cambiado a 'mail' para coincidir con el backend
        passwordHash: hashedPassword, // Cambiado a 'passwordHash' para coincidir con el backend
      },
      true,
    )

    console.log("Registration response received:", response)
    return response
  } catch (error) {
    console.error("Registration error:", error)
    throw error
  }
}

// Función para obtener datos del usuario después del login
export const fetchUserData = async (userId) => {
  try {
    const normalizedId = normalizeId(userId)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid user ID format")
    }

    return apiRequest(`/private/users/${normalizedId}`, "GET")
  } catch (error) {
    console.error("Error fetching user data:", error)
    throw error
  }
}

// Grupos - Actualizado según los nuevos requisitos
export const fetchUserGroups = async (userId) => {
  try {
    const normalizedId = normalizeId(userId)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid user ID format")
    }

    // Nueva llamada para obtener los grupos de un usuario específico
    return apiRequest(`/private/user-group/user/${normalizedId}`, "GET")
  } catch (error) {
    console.error("Error fetching user groups:", error)
    throw error
  }
}

export const createGroup = async (data) => {
  try {
    // Preparar los datos según la estructura requerida del esquema de validación
    const groupData = {
      name: data.name,
      description: "", // Mantener vacío pero incluirlo
      userCount: 0, // Inicializar a 0
      userMax: data.userMax || null, // Opcional
      groupCode: generateRandomCode(8), // Código de grupo aleatorio de 8 caracteres
      tags: data.tags || [], // Opcional
    }

    // La API devuelve solo el ID del grupo creado, no el objeto completo
    const response = await apiRequest("/private/groups", "POST", groupData)

    console.log("Create group response:", response, "Type:", typeof response)

    // La respuesta puede ser directamente el objeto con $oid
    if (response && typeof response === "object") {
      // Normalizar y devolver el ID del grupo creado
      const groupId = normalizeId(response)
      console.log("Normalized group ID:", groupId)

      if (!isValidId(groupId)) {
        throw new Error("Failed to create group: No valid ID returned from API")
      }

      return { _id: groupId }
    }

    throw new Error("Invalid response format from API")
  } catch (error) {
    console.error("Error creating group:", error)
    throw error
  }
}

// Función para generar un código aleatorio de n caracteres
function generateRandomCode(length) {
  const characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
  let result = ""
  for (let i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * characters.length))
  }
  return result
}

export const fetchGroupById = async (groupId) => {
  try {
    const normalizedId = normalizeId(groupId)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid group ID format")
    }

    return apiRequest(`/private/groups/${normalizedId}`, "GET")
  } catch (error) {
    console.error("Error fetching group:", error)
    throw error
  }
}

export const checkGroupByCode = async (code) => {
  try {
    // Verificar si existe un grupo con el código proporcionado
    return apiRequest(`/private/groups/code/${code}`, "GET")
  } catch (error) {
    console.error("Error checking group code:", error)
    throw error
  }
}

export const joinGroup = async (groupId, userId) => {
  try {
    const normalizedGroupId = normalizeId(groupId)
    const normalizedUserId = normalizeId(userId)

    if (!isValidId(normalizedGroupId) || !isValidId(normalizedUserId)) {
      throw new Error("Invalid ID format for group or user")
    }

    // Crear una relación usuario-grupo
    // La API devuelve el ID de la relación creada
    const response = await apiRequest("/private/user-group", "POST", {
      groupId: normalizedGroupId,
      userId: normalizedUserId,
    })

    console.log("Join group response:", response, "Type:", typeof response)

    // La respuesta puede ser directamente el objeto con $oid
    if (response && typeof response === "object") {
      // Normalizar y devolver el ID de la relación creada
      const relationshipId = normalizeId(response)
      console.log("Normalized relationship ID:", relationshipId)

      if (!isValidId(relationshipId)) {
        throw new Error("Failed to join group: No valid relationship ID returned from API")
      }

      return { _id: relationshipId }
    }

    throw new Error("Invalid response format from API")
  } catch (error) {
    console.error("Error joining group:", error)
    throw error
  }
}

export const getUserGroupRelationship = async (relationshipId) => {
  try {
    const normalizedId = normalizeId(relationshipId)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid relationship ID format")
    }

    console.log("Fetching user-group relationship with ID:", normalizedId)

    // Obtener detalles de la relación usuario-grupo
    const response = await apiRequest(`/private/user-group/${normalizedId}`, "GET")
    console.log("User-group relationship response:", response)

    return response
  } catch (error) {
    console.error("Error fetching user-group relationship:", error)
    throw error
  }
}

export const checkUserInGroup = async (groupId, userId) => {
  try {
    const normalizedGroupId = normalizeId(groupId)
    const normalizedUserId = normalizeId(userId)

    if (!isValidId(normalizedGroupId) || !isValidId(normalizedUserId)) {
      return false
    }

    // Verificar si el usuario pertenece al grupo
    const result = await apiRequest(`/private/user-group/check/${normalizedUserId}/${normalizedGroupId}`, "GET")
    return !!result
  } catch (error) {
    console.error("Error checking user in group:", error)
    return false
  }
}

export const updateGroup = async (groupId, groupData) => {
  try {
    const normalizedId = normalizeId(groupId)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid group ID format")
    }

    // Asegurarse de que el ID en los datos también esté normalizado
    if (groupData._id) {
      groupData._id = normalizedId
    }

    // Asegurarse de que description esté presente aunque sea vacío
    if (!groupData.hasOwnProperty("description")) {
      groupData.description = ""
    }

    // Actualizar un grupo existente
    const response = await apiRequest(`/private/groups/${normalizedId}`, "PUT", groupData)

    // Si la respuesta es un objeto con $oid, extraer el ID
    if (response && typeof response === "object" && response.$oid) {
      return { _id: response.$oid }
    }

    // Si la respuesta ya tiene _id o id, usarlo
    if (response && (response._id || response.id)) {
      return { _id: normalizeId(response._id || response.id) }
    }

    // Si la respuesta es el ID directamente
    if (response && typeof response === "string") {
      return { _id: response }
    }

    // Si llegamos aquí, asumimos que la actualización fue exitosa pero no tenemos un ID
    return { _id: normalizedId }
  } catch (error) {
    console.error("Error updating group:", error)
    throw error
  }
}

export const deleteGroup = async (groupId) => {
  try {
    const normalizedId = normalizeId(groupId)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid group ID format")
    }

    // Eliminar un grupo
    return apiRequest(`/private/groups/${normalizedId}`, "DELETE")
  } catch (error) {
    console.error("Error deleting group:", error)
    throw error
  }
}

// Función para obtener el ID de la relación usuario-grupo
export const getUserGroupId = async (userId, groupId) => {
  try {
    const normalizedUserId = normalizeId(userId)
    const normalizedGroupId = normalizeId(groupId)

    if (!isValidId(normalizedUserId) || !isValidId(normalizedGroupId)) {
      throw new Error("Invalid ID format for user or group")
    }

    // Obtener el ID de la relación usuario-grupo
    const response = await apiRequest(`/private/user-group/id/${normalizedUserId}/${normalizedGroupId}`, "GET")

    if (!response) {
      throw new Error("Failed to get user-group ID")
    }

    // Extraer el ID de la respuesta
    const userGroupId = normalizeId(response)

    if (!isValidId(userGroupId)) {
      throw new Error("Invalid user-group ID returned from API")
    }

    return userGroupId
  } catch (error) {
    console.error("Error getting user-group ID:", error)
    throw error
  }
}

// Función para eliminar una relación usuario-grupo
export const deleteUserGroup = async (userGroupId) => {
  try {
    const normalizedId = normalizeId(userGroupId)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid user-group ID format")
    }

    // Eliminar la relación usuario-grupo
    return apiRequest(`/private/user-group/${normalizedId}`, "DELETE")
  } catch (error) {
    console.error("Error deleting user-group:", error)
    throw error
  }
}

// Actualizar la función leaveGroup para usar los nuevos endpoints
export const leaveGroup = async (groupId, userId) => {
  try {
    const normalizedGroupId = normalizeId(groupId)
    const normalizedUserId = normalizeId(userId)

    if (!isValidId(normalizedGroupId) || !isValidId(normalizedUserId)) {
      throw new Error("Invalid ID format for group or user")
    }

    // 1. Verificar primero si el usuario pertenece al grupo
    const isUserInGroup = await checkUserInGroup(normalizedGroupId, normalizedUserId)
    if (!isUserInGroup) {
      console.log("User is not in this group, cannot leave")
      throw new Error("User is not in this group")
    }

    // 2. Obtener el ID de la relación usuario-grupo
    const response = await apiRequest(`/private/user-group/id/${normalizedUserId}/${normalizedGroupId}`, "GET")

    if (!response) {
      throw new Error("No user-group relationship found")
    }

    // Extraer el ID de la respuesta
    const userGroupId = normalizeId(response)

    if (!userGroupId) {
      throw new Error("Invalid user-group ID returned from API")
    }

    // 3. Eliminar la relación usuario-grupo
    await apiRequest(`/private/user-group/${userGroupId}`, "DELETE")

    // 4. Actualizar el contador de usuarios del grupo
    const groupDetails = await fetchGroupById(normalizedGroupId)
    if (groupDetails) {
      const updatedGroup = {
        ...groupDetails,
        userCount: Math.max(0, (groupDetails.userCount || 1) - 1),
      }

      // Actualizar el grupo sin verificar pertenencia (ya que acabamos de eliminar la relación)
      await apiRequest(`/private/groups/${normalizedGroupId}`, "PUT", updatedGroup)
    }

    return { success: true }
  } catch (error) {
    console.error("Error leaving group:", error)
    throw error
  }
}

// Propiedades - Using /private prefix
export const fetchProperties = async (groupId) => {
  try {
    const normalizedId = normalizeId(groupId)
    if (!isValidId(normalizedId)) {
      return []
    }

    return apiRequest(`/private/properties/group/${normalizedId}`, "GET")
  } catch (error) {
    console.error("Error fetching properties:", error)
    return []
  }
}

export const fetchPropertyById = async (propertyId) => {
  try {
    const normalizedId = normalizeId(propertyId)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid property ID format")
    }

    return apiRequest(`/private/properties/${normalizedId}`, "GET")
  } catch (error) {
    console.error("Error fetching property:", error)
    throw error
  }
}

export const createProperty = async (groupId, data) => {
  try {
    const normalizedGroupId = normalizeId(groupId)
    if (!isValidId(normalizedGroupId)) {
      throw new Error("Invalid group ID format")
    }

    // Ensure data matches the properties schema
    const propertyData = {
      name: data.name,
      description: data.description || "",
      direction: data.address || "", // Cambiado a 'direction' según el esquema
      groupId: normalizedGroupId,
      userId: data.userId || null, // Opcional
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }

    // La API devuelve solo el ID de la propiedad creada
    const response = await apiRequest("/private/properties", "POST", propertyData)

    // Verificar si la respuesta contiene un ID
    if (!response || (!response._id && !response.id && !response.insertedId)) {
      throw new Error("Failed to create property: No ID returned from API")
    }

    // Normalizar y devolver el ID de la propiedad creada
    const propertyId = normalizeId(response._id || response.id || response.insertedId)
    console.log("Property created with ID:", propertyId)

    return { _id: propertyId }
  } catch (error) {
    console.error("Error creating property:", error)
    throw error
  }
}

export const updateProperty = async (id, data) => {
  try {
    const normalizedId = normalizeId(id)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid property ID format")
    }

    // Ensure data matches the properties schema
    const propertyData = {
      ...data,
      updated_at: new Date().toISOString(),
    }

    // La API devuelve solo el ID de la propiedad actualizada
    const response = await apiRequest(`/private/properties/${normalizedId}`, "PUT", propertyData)

    // Verificar si la respuesta contiene un ID
    if (!response || (!response._id && !response.id)) {
      throw new Error("Failed to update property: No ID returned from API")
    }

    // Normalizar y devolver el ID de la propiedad actualizada
    const updatedPropertyId = normalizeId(response._id || response.id)
    console.log("Property updated with ID:", updatedPropertyId)

    return { _id: updatedPropertyId }
  } catch (error) {
    console.error("Error updating property:", error)
    throw error
  }
}

export const deleteProperty = async (id) => {
  try {
    const normalizedId = normalizeId(id)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid property ID format")
    }

    return apiRequest(`/private/properties/${normalizedId}`, "DELETE")
  } catch (error) {
    console.error("Error deleting property:", error)
    throw error
  }
}

// Zonas - Using /private prefix
export const fetchZones = async (propertyId) => {
  try {
    const normalizedId = normalizeId(propertyId)
    if (!isValidId(normalizedId)) {
      return []
    }

    // Obtener zonas asociadas a una propiedad
    return apiRequest(`/private/zones/property/${normalizedId}`, "GET")
  } catch (error) {
    console.error("Error fetching zones:", error)
    return []
  }
}

export const fetchZoneById = async (zoneId) => {
  try {
    const normalizedId = normalizeId(zoneId)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid zone ID format")
    }

    return apiRequest(`/private/zones/${normalizedId}`, "GET")
  } catch (error) {
    console.error("Error fetching zone:", error)
    throw error
  }
}

export const fetchSubZones = async (zoneId) => {
  try {
    const normalizedId = normalizeId(zoneId)
    if (!isValidId(normalizedId)) {
      return []
    }

    // Obtener subzonas asociadas a una zona padre
    return apiRequest(`/private/zones/parent/${normalizedId}`, "GET")
  } catch (error) {
    console.error("Error fetching subzones:", error)
    return []
  }
}

export const createZone = async (propertyId, parentZoneId = null, data) => {
  try {
    const normalizedPropertyId = normalizeId(propertyId)
    if (!isValidId(normalizedPropertyId)) {
      throw new Error("Invalid property ID format")
    }

    // Normalizar el ID de la zona padre si existe
    let normalizedParentZoneId = null
    if (parentZoneId) {
      normalizedParentZoneId = normalizeId(parentZoneId)
      if (!isValidId(normalizedParentZoneId)) {
        throw new Error("Invalid parent zone ID format")
      }
    }

    // Ensure data matches the zones schema
    const zoneData = {
      name: data.name,
      description: data.description || "",
      propertyId: normalizedPropertyId,
      parentZoneId: normalizedParentZoneId,
      userId: data.userId || null, // Opcional
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }

    // La API devuelve solo el ID de la zona creada
    const response = await apiRequest("/private/zones", "POST", zoneData)

    // Verificar si la respuesta contiene un ID
    if (!response || (!response._id && !response.id && !response.insertedId)) {
      throw new Error("Failed to create zone: No ID returned from API")
    }

    // Normalizar y devolver el ID de la zona creada
    const zoneId = normalizeId(response._id || response.id || response.insertedId)
    console.log("Zone created with ID:", zoneId)

    return { _id: zoneId }
  } catch (error) {
    console.error("Error creating zone:", error)
    throw error
  }
}

export const updateZone = async (id, data) => {
  try {
    const normalizedId = normalizeId(id)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid zone ID format")
    }

    // Ensure data matches the zones schema
    const zoneData = {
      ...data,
      updated_at: new Date().toISOString(),
    }

    // La API devuelve solo el ID de la zona actualizada
    const response = await apiRequest(`/private/zones/${normalizedId}`, "PUT", zoneData)

    // Verificar si la respuesta contiene un ID
    if (!response || (!response._id && !response.id)) {
      throw new Error("Failed to update zone: No ID returned from API")
    }

    // Normalizar y devolver el ID de la zona actualizada
    const updatedZoneId = normalizeId(response._id || response.id)
    console.log("Zone updated with ID:", updatedZoneId)

    return { _id: updatedZoneId }
  } catch (error) {
    console.error("Error updating zone:", error)
    throw error
  }
}

export const deleteZone = async (id) => {
  try {
    const normalizedId = normalizeId(id)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid zone ID format")
    }

    return apiRequest(`/private/zones/${normalizedId}`, "DELETE")
  } catch (error) {
    console.error("Error deleting zone:", error)
    throw error
  }
}

// Objetos - Using /private prefix
export const fetchItems = async (zoneId) => {
  try {
    const normalizedId = normalizeId(zoneId)
    if (!isValidId(normalizedId)) {
      return []
    }

    // Obtener items asociados a una zona
    return apiRequest(`/private/items/zone/${normalizedId}`, "GET")
  } catch (error) {
    console.error("Error fetching items:", error)
    return []
  }
}

export const fetchItemById = async (itemId) => {
  try {
    const normalizedId = normalizeId(itemId)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid item ID format")
    }

    return apiRequest(`/private/items/${normalizedId}`, "GET")
  } catch (error) {
    console.error("Error fetching item:", error)
    throw error
  }
}

export const createItem = async (zoneId, data) => {
  try {
    const normalizedZoneId = normalizeId(zoneId)
    if (!isValidId(normalizedZoneId)) {
      throw new Error("Invalid zone ID format")
    }

    // Ensure data matches the items schema
    const itemData = {
      name: data.name,
      description: data.description || "",
      pictureUrl: data.pictureUrl || null, // Opcional
      zoneId: normalizedZoneId,
      values: data.values || [], // Opcional
      tags: data.tags || [], // Opcional
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }

    // La API devuelve solo el ID del item creado
    const response = await apiRequest("/private/items", "POST", itemData)

    // Verificar si la respuesta contiene un ID
    if (!response || (!response._id && !response.id && !response.insertedId)) {
      throw new Error("Failed to create item: No ID returned from API")
    }

    // Normalizar y devolver el ID del item creado
    const itemId = normalizeId(response._id || response.id || response.insertedId)
    console.log("Item created with ID:", itemId)

    return { _id: itemId }
  } catch (error) {
    console.error("Error creating item:", error)
    throw error
  }
}

export const updateItem = async (id, data) => {
  try {
    const normalizedId = normalizeId(id)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid item ID format")
    }

    // Ensure data matches the items schema
    const itemData = {
      ...data,
      updated_at: new Date().toISOString(),
    }

    // La API devuelve solo el ID del item actualizado
    const response = await apiRequest(`/private/items/${normalizedId}`, "PUT", itemData)

    // Verificar si la respuesta contiene un ID
    if (!response || (!response._id && !response.id)) {
      throw new Error("Failed to update item: No ID returned from API")
    }

    // Normalizar y devolver el ID del item actualizado
    const updatedItemId = normalizeId(response._id || response.id)
    console.log("Item updated with ID:", updatedItemId)

    return { _id: updatedItemId }
  } catch (error) {
    console.error("Error updating item:", error)
    throw error
  }
}

export const deleteItem = async (id) => {
  try {
    const normalizedId = normalizeId(id)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid item ID format")
    }

    return apiRequest(`/private/items/${normalizedId}`, "DELETE")
  } catch (error) {
    console.error("Error deleting item:", error)
    throw error
  }
}

// Usuario - Using /private prefix
export const updateUser = (data) => {
  const normalizedId = normalizeId(data.id)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid user ID format"))
  }

  // Ensure data matches the users schema
  const userData = {
    name: data.name,
    email: data.email,
    updated_at: new Date().toISOString(),
  }
  return apiRequest(`/private/users/update/${normalizedId}`, "PUT", userData)
}

export const changePassword = async (userId, oldPassword, newPassword) => {
  try {
    const normalizedId = normalizeId(userId)
    if (!isValidId(normalizedId)) {
      throw new Error("Invalid user ID format")
    }

    console.log("Password change attempt for user:", normalizedId)

    // Hash both old and new passwords before sending
    const hashedOldPassword = await hashPassword(oldPassword)
    const hashedNewPassword = await hashPassword(newPassword)
    console.log("Passwords hashed successfully")

    return apiRequest(`/private/users/password/${normalizedId}`, "PUT", {
      old_password: hashedOldPassword,
      new_password: hashedNewPassword,
    })
  } catch (error) {
    console.error("Password change error:", error)
    throw error
  }
}

export const deleteUser = (userId) => {
  const normalizedId = normalizeId(userId)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid user ID format"))
  }

  return apiRequest(`/private/users/delete/${normalizedId}`, "DELETE")
}

// Admin - Using /private prefix
export const fetchLogs = () => {
  return apiRequest("/private/logs/all", "GET")
}

export const fetchUsers = () => {
  return apiRequest("/private/users/all", "GET")
}

export const updateUserAdmin = (id, data) => {
  const normalizedId = normalizeId(id)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid user ID format"))
  }

  // Ensure data matches the users schema
  const userData = {
    name: data.name,
    email: data.email,
    role: data.role || "user",
    updated_at: new Date().toISOString(),
  }
  return apiRequest(`/private/users/update/${normalizedId}`, "PUT", userData)
}

export const deleteUserAdmin = (id) => {
  const normalizedId = normalizeId(id)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid user ID format"))
  }

  return apiRequest(`/private/users/delete/${normalizedId}`, "DELETE")
}

// Search - Using /private prefix
export const searchItems = (query) => {
  return apiRequest(`/private/items/search/${query}`, "GET")
}

// Export/Import - Using /private prefix
export const exportData = (groupId) => {
  const normalizedId = normalizeId(groupId)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid group ID format"))
  }

  return apiRequest(`/private/groups/export/${normalizedId}`, "GET")
}

export const importData = (groupId, data) => {
  const normalizedId = normalizeId(groupId)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid group ID format"))
  }

  return apiRequest(`/private/groups/import/${normalizedId}`, "POST", data)
}

export const fetchGroups = async () => {
  try {
    // Assuming there's an endpoint to fetch all groups
    return apiRequest("/private/groups", "GET")
  } catch (error) {
    console.error("Error fetching groups:", error)
    throw error
  }
}
