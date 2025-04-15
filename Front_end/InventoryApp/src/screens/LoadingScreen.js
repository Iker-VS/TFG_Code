"use client"

import { useContext } from "react"
import { View, ActivityIndicator, StyleSheet } from "react-native"
import { ThemeContext } from "../context/ThemeContext"

const LoadingScreen = () => {
  const { theme } = useContext(ThemeContext)

  return (
    <View style={[styles.container, { backgroundColor: theme.background }]}>
      <ActivityIndicator size="large" color={theme.primary} />
    </View>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: "center",
    alignItems: "center",
  },
})

export default LoadingScreen
