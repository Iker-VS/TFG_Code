"use client"

import { useState, useContext } from "react"
import { View, Text, StyleSheet, TouchableOpacity, Alert, Modal, ActivityIndicator } from "react-native"
import { Ionicons } from "@expo/vector-icons"
import { ThemeContext } from "../context/ThemeContext"
import { AuthContext } from "../context/AuthContext"
import EntityForm from "../components/EntityForm"
import { updateUser, deleteUser } from "../services/api"

const UserScreen = () => {
  const { theme } = useContext(ThemeContext)
  const { userData, logout, setUserData } = useContext(AuthContext)

  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState(null)
  const [showForm, setShowForm] = useState(false)

  // Actualizar información del usuario
  const handleUpdateUser = async (formData) => {
    setIsLoading(true)
    setError(null)

    try {
      const response = await updateUser(formData)
      setUserData(response)
      setShowForm(false)
      Alert.alert("Éxito", "Información actualizada correctamente")
    } catch (err) {
      console.error("Error updating user:", err)
      setError("Error al actualizar la información. Intente nuevamente.")
    } finally {
      setIsLoading(false)
    }
  }

  // Eliminar cuenta de usuario
  const handleDeleteUser = () => {
    Alert.alert(
      "Confirmar eliminación",
      "¿Está seguro que desea eliminar su cuenta? Esta acción no se puede deshacer.",
      [
        { text: "No", style: "cancel" },
        {
          text: "Sí",
          style: "destructive",
          onPress: async () => {
            setIsLoading(true)
            setError(null)

            try {
              await deleteUser()
              logout()
            } catch (err) {
              console.error("Error deleting user:", err)
              setError("Error al eliminar la cuenta. Intente nuevamente.")
              setIsLoading(false)
            }
          },
        },
      ],
    )
  }

  if (!userData) {
    return (
      <View style={[styles.container, { backgroundColor: theme.background }]}>
        <ActivityIndicator size="large" color={theme.primary} />
      </View>
    )
  }

  return (
    <View style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={[styles.userCard, { backgroundColor: theme.card, borderColor: theme.border }]}>
        <View style={[styles.userAvatar, { backgroundColor: theme.primary + "30" }]}>
          <Text style={[styles.userInitial, { color: theme.primary }]}>
            {userData.name ? userData.name.charAt(0).toUpperCase() : "U"}
          </Text>
        </View>

        <Text style={[styles.userName, { color: theme.text }]}>{userData.name}</Text>
        <Text style={[styles.userEmail, { color: theme.text + "CC" }]}>{userData.email}</Text>

        {userData.role === "admin" && (
          <View style={[styles.adminBadge, { backgroundColor: theme.primary }]}>
            <Text style={styles.adminBadgeText}>Administrador</Text>
          </View>
        )}

        <View style={styles.userActions}>
          <TouchableOpacity
            style={[styles.userActionButton, { backgroundColor: theme.primary }]}
            onPress={() => setShowForm(true)}
          >
            <Ionicons name="create-outline" size={20} color="#fff" />
            <Text style={styles.userActionButtonText}>Editar</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.userActionButton, { backgroundColor: theme.error }]}
            onPress={handleDeleteUser}
          >
            <Ionicons name="trash-outline" size={20} color="#fff" />
            <Text style={styles.userActionButtonText}>Eliminar</Text>
          </TouchableOpacity>
        </View>
      </View>

      {error && <Text style={[styles.errorText, { color: theme.error }]}>{error}</Text>}

      {/* Modal de formulario */}
      <Modal visible={showForm} transparent={true} animationType="slide" onRequestClose={() => setShowForm(false)}>
        <View style={styles.modalContainer}>
          <View style={[styles.modalContent, { backgroundColor: theme.background }]}>
            <EntityForm
              type="user"
              initialData={userData}
              onSubmit={handleUpdateUser}
              onCancel={() => setShowForm(false)}
            />
          </View>
        </View>
      </Modal>

      {isLoading && (
        <View style={styles.loadingOverlay}>
          <ActivityIndicator size="large" color={theme.primary} />
        </View>
      )}
    </View>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 20,
  },
  userCard: {
    borderRadius: 10,
    padding: 20,
    alignItems: "center",
    borderWidth: 1,
  },
  userAvatar: {
    width: 100,
    height: 100,
    borderRadius: 50,
    justifyContent: "center",
    alignItems: "center",
    marginBottom: 15,
  },
  userInitial: {
    fontSize: 40,
    fontWeight: "bold",
  },
  userName: {
    fontSize: 24,
    fontWeight: "bold",
    marginBottom: 5,
  },
  userEmail: {
    fontSize: 16,
    marginBottom: 15,
  },
  adminBadge: {
    paddingHorizontal: 10,
    paddingVertical: 5,
    borderRadius: 15,
    marginBottom: 15,
  },
  adminBadgeText: {
    color: "#fff",
    fontSize: 12,
    fontWeight: "bold",
  },
  userActions: {
    flexDirection: "row",
    justifyContent: "center",
    width: "100%",
  },
  userActionButton: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "center",
    paddingVertical: 10,
    paddingHorizontal: 15,
    borderRadius: 5,
    marginHorizontal: 5,
  },
  userActionButtonText: {
    color: "#fff",
    marginLeft: 5,
    fontWeight: "bold",
  },
  errorText: {
    marginTop: 20,
    textAlign: "center",
    fontSize: 16,
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
  loadingOverlay: {
    ...StyleSheet.absoluteFillObject,
    backgroundColor: "rgba(0, 0, 0, 0.3)",
    justifyContent: "center",
    alignItems: "center",
  },
})

export default UserScreen
