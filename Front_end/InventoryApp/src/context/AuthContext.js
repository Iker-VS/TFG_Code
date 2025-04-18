"use client"

import { createContext, useState, useEffect } from "react"
import AsyncStorage from "@react-native-async-storage/async-storage"
import { apiLogin, apiRegister } from "../services/api"
import { validatePassword } from "../utils/password"

export const AuthContext = createContext()

export const AuthProvider = ({ children }) => {
  const [isLoading, setIsLoading] = useState(true)
  const [userToken, setUserToken] = useState(null)
  const [userData, setUserData] = useState(null)
  const [error, setError] = useState(null)

  useEffect(() => {
    // Check if user is logged in
    const bootstrapAsync = async () => {
      try {
        const token = await AsyncStorage.getItem("userToken")
        const user = await AsyncStorage.getItem("userData")

        if (token && user) {
          setUserToken(token)
          setUserData(JSON.parse(user))
        }
      } catch (e) {
        console.error("Failed to load auth data", e)
      } finally {
        setIsLoading(false)
      }
    }

    bootstrapAsync()
  }, [])

  // Function to clear error messages
  const clearErrors = () => {
    setError(null)
  }

  // Extract user ID from JWT token
  const extractUserIdFromToken = (token) => {
    try {
      // Simple JWT parsing without library
      const base64Url = token.split(".")[1]
      const base64 = base64Url.replace(/-/g, "+").replace(/_/g, "/")
      const jsonPayload = decodeURIComponent(
        atob(base64)
          .split("")
          .map((c) => "%" + ("00" + c.charCodeAt(0).toString(16)).slice(-2))
          .join(""),
      )

      const payload = JSON.parse(jsonPayload)
      return payload.sub || payload.id || null // Added payload.id as fallback
    } catch (e) {
      console.error("Error extracting user ID from token:", e)
      return null
    }
  }

  // Modificar la función login para manejar la nueva estructura de respuesta
  const login = async (email, password) => {
    setIsLoading(true)
    setError(null)

    try {
      console.log("Attempting login for:", email)

      // apiLogin will hash the password before sending
      const response = await apiLogin(email, password)
      console.log("Login response received:", response)

      if (response && response.token) {
        console.log("Login successful, token received")

        // La respuesta ahora incluye tanto el token como el usuario
        const token = response.token
        let userInfo = response.user || {}

        // Adaptar los campos del usuario para que coincidan con nuestra aplicación
        if (userInfo) {
          // Convertir 'mail' a 'email' y 'admin' a 'role' para mantener consistencia en la app
          userInfo = {
            ...userInfo,
            email: userInfo.mail || email, // Usar mail del backend o el email proporcionado
            role: userInfo.admin ? "admin" : "user", // Convertir admin (bool) a role (string)
          }
        }

        setUserToken(token)
        setUserData(userInfo)

        await AsyncStorage.setItem("userToken", token)
        await AsyncStorage.setItem("userData", JSON.stringify(userInfo))

        console.log("User data saved successfully:", userInfo)
      } else {
        console.log("Login failed: No token in response")
        setError("Credenciales inválidas")
      }
    } catch (e) {
      console.error("Login error:", e)
      setError("Error al iniciar sesión: " + (e.message || "Error desconocido"))
    } finally {
      setIsLoading(false)
    }
  }

  // Modificar la función register para usar los campos correctos
  const register = async (name, email, password, confirmPassword) => {
    setIsLoading(true)
    setError(null)

    // Validate password strength
    if (!validatePassword(password)) {
      setError("La contraseña debe tener al menos 8 caracteres, una mayúscula, una minúscula y un número")
      setIsLoading(false)
      return
    }

    // Validate password confirmation
    if (password !== confirmPassword) {
      setError("Las contraseñas no coinciden")
      setIsLoading(false)
      return
    }

    try {
      console.log("Attempting registration for:", email)

      // apiRegister will hash the password before sending
      const response = await apiRegister(name, email, password)
      console.log("Registration response received:", response)

      if (response && response.token) {
        console.log("Registration successful, token received")

        // La respuesta ahora incluye tanto el token como el usuario
        const token = response.token
        let userInfo = response.user || {}

        // Adaptar los campos del usuario para que coincidan con nuestra aplicación
        if (userInfo) {
          // Convertir 'mail' a 'email' y 'admin' a 'role' para mantener consistencia en la app
          userInfo = {
            ...userInfo,
            email: userInfo.mail || email, // Usar mail del backend o el email proporcionado
            role: userInfo.admin ? "admin" : "user", // Convertir admin (bool) a role (string)
          }
        }

        setUserToken(token)
        setUserData(userInfo)

        await AsyncStorage.setItem("userToken", token)
        await AsyncStorage.setItem("userData", JSON.stringify(userInfo))

        console.log("User data saved successfully:", userInfo)
      } else {
        console.log("Registration failed: No token in response")
        setError("Error al crear la cuenta")
      }
    } catch (e) {
      console.error("Registration error:", e)
      setError("Error al registrarse: " + (e.message || "Error desconocido"))
    } finally {
      setIsLoading(false)
    }
  }

  const logout = async () => {
    setIsLoading(true)
    try {
      await AsyncStorage.removeItem("userToken")
      await AsyncStorage.removeItem("userData")
      setUserToken(null)
      setUserData(null)
    } catch (e) {
      console.error("Error al cerrar sesión", e)
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <AuthContext.Provider
      value={{
        isLoading,
        userToken,
        userData,
        error,
        login,
        register,
        logout,
        setUserData,
        clearErrors,
      }}
    >
      {children}
    </AuthContext.Provider>
  )
}
