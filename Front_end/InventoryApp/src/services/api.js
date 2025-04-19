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
const apiRequest = async (endpoint, method = "GET", data = null, skipAuthCheck = false) => {
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
    // Preparar los datos según la estructura requerida, incluyendo groupCode
    const groupData = {
      name: data.name,
      description: data.description || "",
      userMax: data.userMax || null, // Opcional
      userCount: 0, // Inicializar a 0
      groupCode: "00000000", // Código de grupo por defecto
    }

    // Nueva llamada para crear un grupo
    return apiRequest("/private/groups", "POST", groupData)
  } catch (error) {
    console.error("Error creating group:", error)
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
    return apiRequest("/private/user-group", "POST", {
      groupId: normalizedGroupId,
      userId: normalizedUserId,
    })
  } catch (error) {
    console.error("Error joining group:", error)
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
    // Nota: Este endpoint es hipotético y debería implementarse en el backend
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

    // Actualizar un grupo existente
    return apiRequest(`/private/groups/${normalizedId}`, "PUT", groupData)
  } catch (error) {
    console.error("Error updating group:", error)
    throw error
  }
}

export const leaveGroup = async (groupId, userId) => {
  try {
    const normalizedGroupId = normalizeId(groupId)
    const normalizedUserId = normalizeId(userId)

    if (!isValidId(normalizedGroupId) || !isValidId(normalizedUserId)) {
      throw new Error("Invalid ID format for group or user")
    }

    // Eliminar la relación usuario-grupo
    return apiRequest(`/private/user-group/delete/${normalizedUserId}/${normalizedGroupId}`, "DELETE")
  } catch (error) {
    console.error("Error leaving group:", error)
    throw error
  }
}

// Propiedades - Using /private prefix
export const fetchProperties = (groupId) => {
  const normalizedId = normalizeId(groupId)
  if (!isValidId(normalizedId)) {
    return Promise.resolve([])
  }

  return apiRequest(`/private/properties/group/${normalizedId}`, "GET")
}

export const createProperty = (groupId, data) => {
  const normalizedId = normalizeId(groupId)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid group ID format"))
  }

  // Ensure data matches the properties schema
  const propertyData = {
    name: data.name,
    description: data.description || "",
    address: data.address || "",
    group_id: normalizedId,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
  return apiRequest("/private/properties/create", "POST", propertyData)
}

export const updateProperty = (id, data) => {
  const normalizedId = normalizeId(id)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid property ID format"))
  }

  // Ensure data matches the properties schema
  const propertyData = {
    ...data,
    updated_at: new Date().toISOString(),
  }
  return apiRequest(`/private/properties/update/${normalizedId}`, "PUT", propertyData)
}

export const deleteProperty = (id) => {
  const normalizedId = normalizeId(id)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid property ID format"))
  }

  return apiRequest(`/private/properties/delete/${normalizedId}`, "DELETE")
}

// Zonas - Using /private prefix
export const fetchZones = (propertyId) => {
  const normalizedId = normalizeId(propertyId)
  if (!isValidId(normalizedId)) {
    return Promise.resolve([])
  }

  return apiRequest(`/private/zones/property/${normalizedId}`, "GET")
}

export const fetchSubZones = (zoneId) => {
  const normalizedId = normalizeId(zoneId)
  if (!isValidId(normalizedId)) {
    return Promise.resolve([])
  }

  return apiRequest(`/private/zones/parent/${normalizedId}`, "GET")
}

export const createZone = (parentId, isProperty = false, data) => {
  const normalizedId = normalizeId(parentId)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid parent ID format"))
  }

  // Ensure data matches the zones schema
  const zoneData = {
    name: data.name,
    description: data.description || "",
    property_id: isProperty ? normalizedId : null,
    parent_id: isProperty ? null : normalizedId,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
  return apiRequest("/private/zones/create", "POST", zoneData)
}

export const updateZone = (id, data) => {
  const normalizedId = normalizeId(id)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid zone ID format"))
  }

  // Ensure data matches the zones schema
  const zoneData = {
    ...data,
    updated_at: new Date().toISOString(),
  }
  return apiRequest(`/private/zones/update/${normalizedId}`, "PUT", zoneData)
}

export const deleteZone = (id) => {
  const normalizedId = normalizeId(id)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid zone ID format"))
  }

  return apiRequest(`/private/zones/delete/${normalizedId}`, "DELETE")
}

// Objetos - Using /private prefix
export const fetchItems = (zoneId) => {
  const normalizedId = normalizeId(zoneId)
  if (!isValidId(normalizedId)) {
    return Promise.resolve([])
  }

  return apiRequest(`/private/items/zone/${normalizedId}`, "GET")
}

export const createItem = (zoneId, data) => {
  const normalizedId = normalizeId(zoneId)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid zone ID format"))
  }

  // Ensure data matches the items schema
  const itemData = {
    name: data.name,
    description: data.description || "",
    status: data.status || "Activo",
    zone_id: normalizedId,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
  return apiRequest("/private/items/create", "POST", itemData)
}

export const updateItem = (id, data) => {
  const normalizedId = normalizeId(id)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid item ID format"))
  }

  // Ensure data matches the items schema
  const itemData = {
    ...data,
    updated_at: new Date().toISOString(),
  }
  return apiRequest(`/private/items/update/${normalizedId}`, "PUT", itemData)
}

export const deleteItem = (id) => {
  const normalizedId = normalizeId(id)
  if (!isValidId(normalizedId)) {
    return Promise.reject(new Error("Invalid item ID format"))
  }

  return apiRequest(`/private/items/delete/${normalizedId}`, "DELETE")
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
