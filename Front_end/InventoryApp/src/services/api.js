import AsyncStorage from "@react-native-async-storage/async-storage"
import { hashPassword } from "../utils/password"

// Base URL for the API - Keeping the IP address as requested
const API_URL = "http://192.168.1.141:8000"

// Helper function for API requests with proper error handling
const apiRequest = async (endpoint, method = "GET", data = null) => {
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

    if (!response.ok) {
      const errorText = await response.text()
      throw new Error(errorText || "Error en la petición")
    }

    // Check if the response is for login (contains "login" in the URL)
    const isLoginRequest = endpoint.includes("login")

    // Handle content type
    const contentType = response.headers.get("content-type")
    if (contentType && contentType.includes("application/json")) {
      // JSON response
      const responseData = await response.json()
      return responseData
    } else {
      // Text response (like JWT token)
      const text = await response.text()

      // If this is a login request and we got a text response, it's likely a JWT token
      if (isLoginRequest && text) {
        console.log("Received token response")
        // Return in the format expected by AuthContext
        return {
          token: text,
          user: { id: "user-id" }, // Minimal user object, will be updated later
        }
      }

      // For other text responses
      return { success: true, data: text }
    }
  } catch (error) {
    console.error("API Error:", error)
    throw error
  }
}

// Modificar la función apiLogin para manejar la nueva estructura de respuesta
export const apiLogin = async (email, password) => {
  try {
    console.log("Login attempt for:", email)

    // Hash the password before sending
    const hashedPassword = await hashPassword(password)
    console.log("Password hashed successfully")

    // Using the correct endpoint format for login
    const response = await apiRequest(`/public/users/login/${email}/${hashedPassword}`, "POST")

    // La respuesta ahora debería contener directamente el token y el usuario
    console.log("Login response received:", response)

    return response
  } catch (error) {
    console.error("Login error:", error)
    throw error
  }
}

// Actualizar fetchUserData para usar 'mail' en lugar de 'email'
export const fetchUserData = async (userId) => {
  try {
    const userData = await apiRequest(`/private/users/${userId}`, "GET")
    // Adaptar los campos si es necesario
    return userData
  } catch (error) {
    console.error("Error fetching user data:", error)
    throw error
  }
}

// Actualizar apiRegister para usar 'mail' en lugar de 'email'
export const apiRegister = async (name, email, password) => {
  try {
    console.log("Registration attempt for:", email)

    // Hash the password before sending
    const hashedPassword = await hashPassword(password)
    console.log("Password hashed successfully")

    // Usar los nombres de campo correctos según la estructura del backend
    return apiRequest("/public/users/register", "POST", {
      name,
      mail: email, // Cambiado de 'email' a 'mail' para coincidir con el backend
      passwordHash: hashedPassword,
    })
  } catch (error) {
    console.error("Registration error:", error)
    throw error
  }
}

// Grupos - Using /private prefix
export const fetchGroups = () => {
  return apiRequest("/private/groups/all", "GET")
}

export const createGroup = (data) => {
  // Ensure data matches the groups schema
  const groupData = {
    name: data.name,
    description: data.description || "",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
  return apiRequest("/private/groups/create", "POST", groupData)
}

export const updateGroup = (id, data) => {
  // Ensure data matches the groups schema
  const groupData = {
    ...data,
    updated_at: new Date().toISOString(),
  }
  return apiRequest(`/private/groups/update/${id}`, "PUT", groupData)
}

export const deleteGroup = (id) => {
  return apiRequest(`/private/groups/delete/${id}`, "DELETE")
}

export const joinGroup = (code) => {
  return apiRequest(`/private/groups/join/${code}`, "POST")
}

export const leaveGroup = (id) => {
  return apiRequest(`/private/groups/leave/${id}`, "POST")
}

// Propiedades - Using /private prefix
export const fetchProperties = (groupId) => {
  return apiRequest(`/private/properties/group/${groupId}`, "GET")
}

export const createProperty = (groupId, data) => {
  // Ensure data matches the properties schema
  const propertyData = {
    name: data.name,
    description: data.description || "",
    address: data.address || "",
    group_id: groupId,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
  return apiRequest("/private/properties/create", "POST", propertyData)
}

export const updateProperty = (id, data) => {
  // Ensure data matches the properties schema
  const propertyData = {
    ...data,
    updated_at: new Date().toISOString(),
  }
  return apiRequest(`/private/properties/update/${id}`, "PUT", propertyData)
}

export const deleteProperty = (id) => {
  return apiRequest(`/private/properties/delete/${id}`, "DELETE")
}

// Zonas - Using /private prefix
export const fetchZones = (propertyId) => {
  return apiRequest(`/private/zones/property/${propertyId}`, "GET")
}

export const fetchSubZones = (zoneId) => {
  return apiRequest(`/private/zones/parent/${zoneId}`, "GET")
}

export const createZone = (parentId, isProperty = false, data) => {
  // Ensure data matches the zones schema
  const zoneData = {
    name: data.name,
    description: data.description || "",
    property_id: isProperty ? parentId : null,
    parent_id: isProperty ? null : parentId,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
  return apiRequest("/private/zones/create", "POST", zoneData)
}

export const updateZone = (id, data) => {
  // Ensure data matches the zones schema
  const zoneData = {
    ...data,
    updated_at: new Date().toISOString(),
  }
  return apiRequest(`/private/zones/update/${id}`, "PUT", zoneData)
}

export const deleteZone = (id) => {
  return apiRequest(`/private/zones/delete/${id}`, "DELETE")
}

// Objetos - Using /private prefix
export const fetchItems = (zoneId) => {
  return apiRequest(`/private/items/zone/${zoneId}`, "GET")
}

export const createItem = (zoneId, data) => {
  // Ensure data matches the items schema
  const itemData = {
    name: data.name,
    description: data.description || "",
    status: data.status || "Activo",
    zone_id: zoneId,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
  return apiRequest("/private/items/create", "POST", itemData)
}

export const updateItem = (id, data) => {
  // Ensure data matches the items schema
  const itemData = {
    ...data,
    updated_at: new Date().toISOString(),
  }
  return apiRequest(`/private/items/update/${id}`, "PUT", itemData)
}

export const deleteItem = (id) => {
  return apiRequest(`/private/items/delete/${id}`, "DELETE")
}

// Usuario - Using /private prefix
export const updateUser = (data) => {
  // Ensure data matches the users schema
  const userData = {
    name: data.name,
    email: data.email,
    updated_at: new Date().toISOString(),
  }
  return apiRequest(`/private/users/update/${data.id}`, "PUT", userData)
}

export const changePassword = async (userId, oldPassword, newPassword) => {
  try {
    console.log("Password change attempt for user:", userId)

    // Hash both old and new passwords before sending
    const hashedOldPassword = await hashPassword(oldPassword)
    const hashedNewPassword = await hashPassword(newPassword)
    console.log("Passwords hashed successfully")

    return apiRequest(`/private/users/password/${userId}`, "PUT", {
      old_password: hashedOldPassword,
      new_password: hashedNewPassword,
    })
  } catch (error) {
    console.error("Password change error:", error)
    throw error
  }
}

export const deleteUser = (userId) => {
  return apiRequest(`/private/users/delete/${userId}`, "DELETE")
}

// Admin - Using /private prefix
export const fetchLogs = () => {
  return apiRequest("/private/logs/all", "GET")
}

export const fetchUsers = () => {
  return apiRequest("/private/users/all", "GET")
}

export const updateUserAdmin = (id, data) => {
  // Ensure data matches the users schema
  const userData = {
    name: data.name,
    email: data.email,
    role: data.role || "user",
    updated_at: new Date().toISOString(),
  }
  return apiRequest(`/private/users/update/${id}`, "PUT", userData)
}

export const deleteUserAdmin = (id) => {
  return apiRequest(`/private/users/delete/${id}`, "DELETE")
}

// Search - Using /private prefix
export const searchItems = (query) => {
  return apiRequest(`/private/items/search/${query}`, "GET")
}

// Export/Import - Using /private prefix
export const exportData = (groupId) => {
  return apiRequest(`/private/groups/export/${groupId}`, "GET")
}

export const importData = (groupId, data) => {
  return apiRequest(`/private/groups/import/${groupId}`, "POST", data)
}
