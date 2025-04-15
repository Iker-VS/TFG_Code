"use client"

import { createContext, useState, useEffect } from "react"
import AsyncStorage from "@react-native-async-storage/async-storage"
import { apiLogin, apiRegister } from "../services/api"

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

  const login = async (email, password) => {
    setIsLoading(true)
    setError(null)

    try {
      const response = await apiLogin(email, password)

      if (response.token) {
        setUserToken(response.token)
        setUserData(response.user)

        await AsyncStorage.setItem("userToken", response.token)
        await AsyncStorage.setItem("userData", JSON.stringify(response.user))
      } else {
        setError("Credenciales inválidas")
      }
    } catch (e) {
      setError("Error al iniciar sesión")
      console.error(e)
    } finally {
      setIsLoading(false)
    }
  }

  const register = async (name, email, password) => {
    setIsLoading(true)
    setError(null)

    try {
      const response = await apiRegister(name, email, password)

      if (response.token) {
        setUserToken(response.token)
        setUserData(response.user)

        await AsyncStorage.setItem("userToken", response.token)
        await AsyncStorage.setItem("userData", JSON.stringify(response.user))
      } else {
        setError("Error al crear la cuenta")
      }
    } catch (e) {
      setError("Error al registrarse")
      console.error(e)
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
      }}
    >
      {children}
    </AuthContext.Provider>
  )
}
