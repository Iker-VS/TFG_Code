"use client"

import { useState, useEffect, useContext, useCallback } from "react"
import { View, Text, StyleSheet, FlatList, TouchableOpacity, ActivityIndicator } from "react-native"
import { useFocusEffect } from "@react-navigation/native"
import { ThemeContext } from "../context/ThemeContext"
import { AuthContext } from "../context/AuthContext"
import { fetchLogs } from "../services/api"

const LogsScreen = () => {
  const { theme } = useContext(ThemeContext)
  const { userToken, userData } = useContext(AuthContext)

  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState(null)
  const [logs, setLogs] = useState([])

  // Verificar si el usuario es administrador
  useEffect(() => {
    if (userData && userData.role !== "admin") {
      setError("No tienes permisos para acceder a esta sección")
      setIsLoading(false)
    }
  }, [userData])

  // Cargar logs al enfocar la pantalla
  useFocusEffect(
    useCallback(() => {
      if (userToken && userData?.role === "admin") {
        loadLogs()
      }
    }, [userToken, userData]),
  )

  // Cargar logs
  const loadLogs = async () => {
    setIsLoading(true)
    setError(null)

    try {
      const response = await fetchLogs()
      setLogs(response)
    } catch (err) {
      console.error("Error loading logs:", err)
      setError("Error al cargar los logs. Intente nuevamente.")
    } finally {
      setIsLoading(false)
    }
  }

  // Formatear fecha
  const formatDate = (dateString) => {
    const date = new Date(dateString)
    return date.toLocaleString()
  }

  // Renderizar un log
  const renderLogItem = ({ item }) => (
    <View style={[styles.logItem, { backgroundColor: theme.card, borderColor: theme.border }]}>
      <View style={styles.logHeader}>
        <Text
          style={[
            styles.logType,
            {
              color: "#fff",
              backgroundColor:
                item.type === "error" ? theme.error : item.type === "warning" ? theme.warning : theme.primary,
            },
          ]}
        >
          {item.type.toUpperCase()}
        </Text>
        <Text style={[styles.logDate, { color: theme.text + "80" }]}>{formatDate(item.timestamp)}</Text>
      </View>
      <Text style={[styles.logUser, { color: theme.text }]}>Usuario: {item.user || "Sistema"}</Text>
      <Text style={[styles.logMessage, { color: theme.text }]}>{item.message}</Text>
      {item.details && <Text style={[styles.logDetails, { color: theme.text + "CC" }]}>{item.details}</Text>}
    </View>
  )

  if (isLoading) {
    return (
      <View style={[styles.container, { backgroundColor: theme.background }]}>
        <ActivityIndicator size="large" color={theme.primary} />
      </View>
    )
  }

  if (error) {
    return (
      <View style={[styles.container, { backgroundColor: theme.background }]}>
        <View style={styles.centerContainer}>
          <Text style={[styles.errorText, { color: theme.error }]}>{error}</Text>
          {userData?.role === "admin" && (
            <TouchableOpacity style={[styles.retryButton, { backgroundColor: theme.primary }]} onPress={loadLogs}>
              <Text style={styles.retryButtonText}>Reintentar</Text>
            </TouchableOpacity>
          )}
        </View>
      </View>
    )
  }

  return (
    <View style={[styles.container, { backgroundColor: theme.background }]}>
      {logs.length === 0 ? (
        <View style={styles.centerContainer}>
          <Text style={[styles.emptyText, { color: theme.text + "CC" }]}>No hay logs para mostrar</Text>
        </View>
      ) : (
        <FlatList
          data={logs}
          keyExtractor={(item, index) => `log-${index}-${item.timestamp}`}
          renderItem={renderLogItem}
          contentContainerStyle={styles.listContainer}
        />
      )}
    </View>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  centerContainer: {
    flex: 1,
    justifyContent: "center",
    alignItems: "center",
    padding: 20,
  },
  errorText: {
    fontSize: 16,
    textAlign: "center",
    marginBottom: 20,
  },
  emptyText: {
    fontSize: 16,
    textAlign: "center",
  },
  retryButton: {
    paddingHorizontal: 20,
    paddingVertical: 10,
    borderRadius: 5,
  },
  retryButtonText: {
    color: "#fff",
    fontSize: 16,
  },
  listContainer: {
    padding: 15,
  },
  logItem: {
    borderRadius: 8,
    borderWidth: 1,
    padding: 15,
    marginBottom: 10,
  },
  logHeader: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: 10,
  },
  logType: {
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 4,
    fontSize: 12,
    fontWeight: "bold",
  },
  logDate: {
    fontSize: 12,
  },
  logUser: {
    fontSize: 14,
    fontWeight: "bold",
    marginBottom: 5,
  },
  logMessage: {
    fontSize: 14,
    marginBottom: 5,
  },
  logDetails: {
    fontSize: 12,
  },
})

export default LogsScreen
