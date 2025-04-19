"use client"

import { createContext, useState, useEffect } from "react"
import AsyncStorage from "@react-native-async-storage/async-storage"
import { apiLogin, apiRegister } from "../services/api"
import { validatePassword } from "../utils/password"
import { Alert } from "react-native"

export const AuthContext = createContext()

export const AuthProvider = ({ children }) => {
  const [isLoading, setIsLoading] = useState(true)
  const [userToken, setUserToken] = useState(null)
  const [userData, setUserData] = useState(null)
  const [error, setError] = useState(null)
  const [registrationSuccess, setRegistrationSuccess] = useState(null)
  const [authInvalidated, setAuthInvalidated] = useState(false)

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

  // Function to check if authentication data exists
  const hasAuthData = () => {
    return !!userToken && !!userData && !!userData._id
  }

  // Function to invalidate authentication and redirect to login
  const invalidateAuth = () => {
    console.log("Invalidating authentication and redirecting to login")
    setAuthInvalidated(true)
    logout()
  }

  // Reset auth invalidated flag (called after navigation to login screen)
  const resetAuthInvalidated = () => {
    setAuthInvalidated(false)
  }

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

        // Limpiar datos de registro exitoso
        setRegistrationSuccess(null)

        // Reset auth invalidated flag
        resetAuthInvalidated()
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

  // Modify the register function to handle successful registration better
  const register = async (name, email, password, confirmPassword) => {
    setIsLoading(true)
    setError(null)

    // Validate password strength
    if (!validatePassword(password)) {
      setError("La contraseña debe tener al menos 8 caracteres, una mayúscula, una minúscula y un número")
      setIsLoading(false)
      return false
    }

    // Validate password confirmation
    if (password !== confirmPassword) {
      setError("Las contraseñas no coinciden")
      setIsLoading(false)
      return false
    }

    try {
      console.log("Attempting registration for:", email)

      // apiRegister will hash the password before sending
      const response = await apiRegister(name, email, password)
      console.log("Registration response received:", response)

      // El backend solo devuelve el ID del usuario al registrarse
      if (response && (response._id || response.id)) {
        console.log("Registration successful, user ID received:", response._id || response.id)

        // Guardar los datos de registro para autocompletar el login
        setRegistrationSuccess({
          email: email,
          password: password,
        })

        // Mostrar mensaje de éxito y redirigir al login
        Alert.alert("Registration successful", "Your account has been created. You can now log in.", [{ text: "OK" }])

        return true
      } else {
        console.log("Registration failed: No user ID in response")
        setError("Error creating account")
        return false
      }
    } catch (e) {
      console.error("Registration error:", e)
      setError("Registration error: " + (e.message || "Unknown error"))
      return false
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
        registrationSuccess,
        authInvalidated,
        login,
        register,
        logout,
        setUserData,
        clearErrors,
        setRegistrationSuccess,
        hasAuthData,
        invalidateAuth,
        resetAuthInvalidated,
      }}
    >
      {children}
    </AuthContext.Provider>
  )
}
