"use client"

import { createContext, useState, useEffect } from "react"
import { useColorScheme } from "react-native"
import AsyncStorage from "@react-native-async-storage/async-storage"

export const ThemeContext = createContext()

// Definición de temas
const themes = {
  light: {
    background: "#FFFFFF",
    text: "#000000",
    primary: "#4A90E2",
    secondary: "#F5F5F5",
    card: "#FFFFFF",
    border: "#E0E0E0",
    error: "#FF3B30",
    success: "#34C759",
    warning: "#FFCC00",
    panel: "#F8F8F8",
    swipeEdit: "#FFCC00",
    swipeDelete: "#FF3B30",
  },
  dark: {
    background: "#121212",
    text: "#FFFFFF",
    primary: "#4A90E2",
    secondary: "#1E1E1E",
    card: "#1E1E1E",
    border: "#333333",
    error: "#FF453A",
    success: "#30D158",
    warning: "#FFD60A",
    panel: "#2C2C2C",
    swipeEdit: "#FFD60A",
    swipeDelete: "#FF453A",
  },
}

export const ThemeProvider = ({ children }) => {
  const deviceTheme = useColorScheme()
  const [themeMode, setThemeMode] = useState("light")
  const [theme, setTheme] = useState(themes.light)

  useEffect(() => {
    // Cargar preferencia de tema guardada
    const loadTheme = async () => {
      try {
        const savedTheme = await AsyncStorage.getItem("themeMode")
        if (savedTheme) {
          setThemeMode(savedTheme)
          setTheme(themes[savedTheme])
        } else {
          // Usar tema del dispositivo por defecto
          setThemeMode(deviceTheme || "light")
          setTheme(themes[deviceTheme] || themes.light)
        }
      } catch (e) {
        console.error("Error al cargar el tema", e)
      }
    }

    loadTheme()
  }, [deviceTheme])

  const toggleTheme = async () => {
    const newThemeMode = themeMode === "light" ? "dark" : "light"
    setThemeMode(newThemeMode)
    setTheme(themes[newThemeMode])

    try {
      await AsyncStorage.setItem("themeMode", newThemeMode)
    } catch (e) {
      console.error("Error al guardar el tema", e)
    }
  }

  return <ThemeContext.Provider value={{ theme, themeMode, toggleTheme }}>{children}</ThemeContext.Provider>
}
