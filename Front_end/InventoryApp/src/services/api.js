import AsyncStorage from "@react-native-async-storage/async-storage"

const API_URL = "http://localhost:8000"

// Función auxiliar para realizar peticiones a la API
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

  if (data) {
    config.body = JSON.stringify(data)
  }

  try {
    const response = await fetch(`${API_URL}${endpoint}`, config)
    const responseData = await response.json()

    if (!response.ok) {
      throw new Error(responseData.message || "Error en la petición")
    }

    return responseData
  } catch (error) {
    console.error("API Error:", error)
    throw error
  }
}

// Autenticación
export const apiLogin = (email, password) => {
  return apiRequest("/auth/login", "POST", { email, password })
}

export const apiRegister = (name, email, password) => {
  return apiRequest("/auth/register", "POST", { name, email, password })
}

// Grupos
export const fetchGroups = () => {
  return apiRequest("/groups")
}

export const createGroup = (data) => {
  return apiRequest("/groups", "POST", data)
}

export const updateGroup = (id, data) => {
  return apiRequest(`/groups/${id}`, "PUT", data)
}

export const deleteGroup = (id) => {
  return apiRequest(`/groups/${id}`, "DELETE")
}

export const joinGroup = (code) => {
  return apiRequest("/groups/join", "POST", { code })
}

export const leaveGroup = (id) => {
  return apiRequest(`/groups/${id}/leave`, "POST")
}

// Propiedades
export const fetchProperties = (groupId) => {
  return apiRequest(`/groups/${groupId}/properties`)
}

export const createProperty = (groupId, data) => {
  return apiRequest(`/groups/${groupId}/properties`, "POST", data)
}

export const updateProperty = (id, data) => {
  return apiRequest(`/properties/${id}`, "PUT", data)
}

export const deleteProperty = (id) => {
  return apiRequest(`/properties/${id}`, "DELETE")
}

// Zonas
export const fetchZones = (propertyId) => {
  return apiRequest(`/properties/${propertyId}/zones`)
}

export const fetchSubZones = (zoneId) => {
  return apiRequest(`/zones/${zoneId}/zones`)
}

export const createZone = (parentId, isProperty = false, data) => {
  const endpoint = isProperty ? `/properties/${parentId}/zones` : `/zones/${parentId}/zones`
  return apiRequest(endpoint, "POST", data)
}

export const updateZone = (id, data) => {
  return apiRequest(`/zones/${id}`, "PUT", data)
}

export const deleteZone = (id) => {
  return apiRequest(`/zones/${id}`, "DELETE")
}

// Objetos
export const fetchItems = (zoneId) => {
  return apiRequest(`/zones/${zoneId}/items`)
}

export const createItem = (zoneId, data) => {
  return apiRequest(`/zones/${zoneId}/items`, "POST", data)
}

export const updateItem = (id, data) => {
  return apiRequest(`/items/${id}`, "PUT", data)
}

export const deleteItem = (id) => {
  return apiRequest(`/items/${id}`, "DELETE")
}

// Usuario
export const updateUser = (data) => {
  return apiRequest("/users/profile", "PUT", data)
}

export const deleteUser = () => {
  return apiRequest("/users/profile", "DELETE")
}

// Admin
export const fetchLogs = () => {
  return apiRequest("/admin/logs")
}

export const fetchUsers = () => {
  return apiRequest("/admin/users")
}

export const updateUserAdmin = (id, data) => {
  return apiRequest(`/admin/users/${id}`, "PUT", data)
}

export const deleteUserAdmin = (id) => {
  return apiRequest(`/admin/users/${id}`, "DELETE")
}
