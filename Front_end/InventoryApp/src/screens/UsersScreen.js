"use client"

import { useState, useEffect, useContext, useCallback } from "react"
import { View, Text, StyleSheet, FlatList, TouchableOpacity, Alert, Modal, ActivityIndicator } from "react-native"
import { useFocusEffect } from "@react-navigation/native"
import { ThemeContext } from "../context/ThemeContext"
import { AuthContext } from "../context/AuthContext"
import EntityForm from "../components/EntityForm"
import SwipeablePanel from "../components/SwipeablePanel"
import { fetchUsers, updateUserAdmin, deleteUserAdmin } from "../services/api"

const UsersScreen = () => {
  const { theme } = useContext(ThemeContext)
  const { userToken, userData } = useContext(AuthContext)

  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState(null)
  const [users, setUsers] = useState([])
  const [showForm, setShowForm] = useState(false)
  const [editingUser, setEditingUser] = useState(null)

  // Verificar si el usuario es administrador
  useEffect(() => {
    if (userData && userData.role !== "admin") {
      setError("No tienes permisos para acceder a esta sección")
      setIsLoading(false)
    }
  }, [userData])

  // Cargar usuarios al enfocar la pantalla
  useFocusEffect(
    useCallback(() => {
      if (userToken && userData?.role === "admin") {
        loadUsers()
      }
    }, [userToken, userData]),
  )

  // Cargar usuarios
  const loadUsers = async () => {
    setIsLoading(true)
    setError(null)

    try {
      const response = await fetchUsers()
      setUsers(response)
    } catch (err) {
      console.error("Error loading users:", err)
      setError("Error al cargar los usuarios. Intente nuevamente.")
    } finally {
      setIsLoading(false)
    }
  }

  // Actualizar un usuario
  const handleUpdateUser = async (formData) => {
    setIsLoading(true)
    setError(null)

    try {
      await updateUserAdmin(editingUser.id, formData)
      setShowForm(false)
      setEditingUser(null)
      loadUsers()
    } catch (err) {
      console.error("Error updating user:", err)
      setError("Error al actualizar el usuario. Intente nuevamente.")
    } finally {
      setIsLoading(false)
    }
  }

  // Eliminar un usuario
  const handleDeleteUser = (user) => {
    // No permitir eliminar el propio usuario
    if (user.id === userData.id) {
      Alert.alert("Error", "No puedes eliminar tu propia cuenta desde aquí")
      return
    }

    Alert.alert("Confirmar eliminación", `¿Está seguro que desea eliminar al usuario ${user.name}?`, [
      { text: "No", style: "cancel" },
      {
        text: "Sí",
        style: "destructive",
        onPress: async () => {
          setIsLoading(true)
          setError(null)

          try {
            await deleteUserAdmin(user.id)
            loadUsers()
          } catch (err) {
            console.error("Error deleting user:", err)
            setError("Error al eliminar el usuario. Intente nuevamente.")
          } finally {
            setIsLoading(false)
          }
        },
      },
    ])
  }

  // Renderizar un usuario
  const renderUserItem = ({ item }) => (
    <SwipeablePanel
      onEdit={() => {
        setEditingUser(item)
        setShowForm(true)
      }}
      onDelete={() => handleDeleteUser(item)}
    >
      <View style={[styles.userItem, { backgroundColor: theme.card, borderColor: theme.border }]}>
        <View style={styles.userInfo}>
          <View style={[styles.userAvatar, { backgroundColor: theme.primary + "30" }]}>
            <Text style={[styles.userInitial, { color: theme.primary }]}>
              {item.name ? item.name.charAt(0).toUpperCase() : "U"}
            </Text>
          </View>
          <View style={styles.userDetails}>
            <Text style={[styles.userName, { color: theme.text }]}>{item.name}</Text>
            <Text style={[styles.userEmail, { color: theme.text + "CC" }]}>{item.email}</Text>
          </View>
        </View>
        {item.role === "admin" && (
          <View style={[styles.adminBadge, { backgroundColor: theme.primary }]}>
            <Text style={styles.adminBadgeText}>Admin</Text>
          </View>
        )}
      </View>
    </SwipeablePanel>
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
            <TouchableOpacity style={[styles.retryButton, { backgroundColor: theme.primary }]} onPress={loadUsers}>
              <Text style={styles.retryButtonText}>Reintentar</Text>
            </TouchableOpacity>
          )}
        </View>
      </View>
    )
  }

  return (
    <View style={[styles.container, { backgroundColor: theme.background }]}>
      {users.length === 0 ? (
        <View style={styles.centerContainer}>
          <Text style={[styles.emptyText, { color: theme.text + "CC" }]}>No hay usuarios para mostrar</Text>
        </View>
      ) : (
        <FlatList
          data={users}
          keyExtractor={(item) => item.id.toString()}
          renderItem={renderUserItem}
          contentContainerStyle={styles.listContainer}
        />
      )}

      {/* Modal de formulario */}
      <Modal
        visible={showForm}
        transparent={true}
        animationType="slide"
        onRequestClose={() => {
          setShowForm(false)
          setEditingUser(null)
        }}
      >
        <View style={styles.modalContainer}>
          <View style={[styles.modalContent, { backgroundColor: theme.background }]}>
            <EntityForm
              type="user"
              initialData={editingUser || {}}
              onSubmit={handleUpdateUser}
              onCancel={() => {
                setShowForm(false)
                setEditingUser(null)
              }}
            />
          </View>
        </View>
      </Modal>
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
  userItem: {
    borderRadius: 8,
    borderWidth: 1,
    padding: 15,
    marginBottom: 10,
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
  },
  userInfo: {
    flexDirection: "row",
    alignItems: "center",
  },
  userAvatar: {
    width: 40,
    height: 40,
    borderRadius: 20,
    justifyContent: "center",
    alignItems: "center",
    marginRight: 10,
  },
  userInitial: {
    fontSize: 18,
    fontWeight: "bold",
  },
  userDetails: {
    flex: 1,
  },
  userName: {
    fontSize: 16,
    fontWeight: "bold",
  },
  userEmail: {
    fontSize: 14,
  },
  adminBadge: {
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 4,
  },
  adminBadgeText: {
    color: "#fff",
    fontSize: 12,
    fontWeight: "bold",
  },
  modalContainer: {
    flex: 1,
    justifyContent: "center",
    backgroundColor: "rgba(0, 0, 0, 0.5)",
  },
  modalContent: {
    flex: 1,
    margin: 20,
    borderRadius: 10,
    overflow: "hidden",
  },
})

export default UsersScreen
