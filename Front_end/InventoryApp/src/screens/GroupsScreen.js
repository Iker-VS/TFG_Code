"use client"

import { useState, useContext, useCallback } from "react"
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  Alert,
  Modal,
  TextInput,
  ActivityIndicator,
} from "react-native"
import { useFocusEffect } from "@react-navigation/native"
import { Ionicons } from "@expo/vector-icons"
import { ThemeContext } from "../context/ThemeContext"
import { AuthContext } from "../context/AuthContext"
import EntityPanel from "../components/EntityPanel"
import SwipeablePanel from "../components/SwipeablePanel"
import EntityForm from "../components/EntityForm"
import { fetchGroups, createGroup, updateGroup, leaveGroup, joinGroup } from "../services/api"

const GroupsScreen = ({ navigation }) => {
  const { theme } = useContext(ThemeContext)
  const { userToken } = useContext(AuthContext)

  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState(null)
  const [groups, setGroups] = useState([])
  const [showForm, setShowForm] = useState(false)
  const [editingGroup, setEditingGroup] = useState(null)
  const [showJoinModal, setShowJoinModal] = useState(false)
  const [joinCode, setJoinCode] = useState("")

  // Cargar datos iniciales al enfocar la pantalla
  useFocusEffect(
    useCallback(() => {
      loadGroups()
    }, [userToken]),
  )

  // Cargar grupos
  const loadGroups = async () => {
    if (!userToken) return

    setIsLoading(true)
    setError(null)

    try {
      const response = await fetchGroups()
      setGroups(response)
    } catch (err) {
      console.error("Error loading groups:", err)
      setError("Error al cargar los grupos. Intente nuevamente.")
    } finally {
      setIsLoading(false)
    }
  }

  // Crear un nuevo grupo
  const handleCreateGroup = async (formData) => {
    setIsLoading(true)
    setError(null)

    try {
      await createGroup(formData)
      setShowForm(false)
      loadGroups()
    } catch (err) {
      console.error("Error creating group:", err)
      setError("Error al crear el grupo. Intente nuevamente.")
    } finally {
      setIsLoading(false)
    }
  }

  // Actualizar un grupo existente
  const handleUpdateGroup = async (formData) => {
    setIsLoading(true)
    setError(null)

    try {
      await updateGroup(editingGroup.id, formData)
      setShowForm(false)
      setEditingGroup(null)
      loadGroups()
    } catch (err) {
      console.error("Error updating group:", err)
      setError("Error al actualizar el grupo. Intente nuevamente.")
    } finally {
      setIsLoading(false)
    }
  }

  // Salir de un grupo
  const handleLeaveGroup = (group) => {
    Alert.alert("Confirmar salida", `¿Está seguro que desea salir del grupo ${group.name}?`, [
      { text: "No", style: "cancel" },
      {
        text: "Sí",
        style: "destructive",
        onPress: async () => {
          setIsLoading(true)
          setError(null)

          try {
            await leaveGroup(group.id)
            loadGroups()
          } catch (err) {
            console.error("Error leaving group:", err)
            setError("Error al salir del grupo. Intente nuevamente.")
          } finally {
            setIsLoading(false)
          }
        },
      },
    ])
  }

  // Unirse a un grupo con código
  const handleJoinGroup = async () => {
    if (!joinCode.trim()) {
      Alert.alert("Error", "Por favor ingrese un código")
      return
    }

    setIsLoading(true)
    setError(null)

    try {
      await joinGroup(joinCode)
      setShowJoinModal(false)
      setJoinCode("")
      loadGroups()
    } catch (err) {
      console.error("Error joining group:", err)
      Alert.alert("Error", "Código inválido o error al unirse al grupo")
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <View style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Lista de grupos */}
      {isLoading ? (
        <View style={styles.centerContainer}>
          <ActivityIndicator size="large" color={theme.primary} />
        </View>
      ) : error ? (
        <View style={styles.centerContainer}>
          <Text style={[styles.errorText, { color: theme.error }]}>{error}</Text>
          <TouchableOpacity style={[styles.retryButton, { backgroundColor: theme.primary }]} onPress={loadGroups}>
            <Text style={styles.retryButtonText}>Reintentar</Text>
          </TouchableOpacity>
        </View>
      ) : groups.length === 0 ? (
        <View style={styles.centerContainer}>
          <Text style={[styles.emptyText, { color: theme.text + "CC" }]}>No perteneces a ningún grupo</Text>
        </View>
      ) : (
        <FlatList
          data={groups}
          keyExtractor={(item) => item.id.toString()}
          contentContainerStyle={styles.listContainer}
          renderItem={({ item }) => (
            <SwipeablePanel
              onEdit={() => {
                setEditingGroup(item)
                setShowForm(true)
              }}
              onDelete={() => handleLeaveGroup(item)}
            >
              <EntityPanel
                entity={item}
                type="group"
                onPress={() => {
                  // Navegar a la pantalla principal con este grupo
                  navigation.navigate("Home", { group: item })
                }}
              />
            </SwipeablePanel>
          )}
        />
      )}

      {/* Botones de acción */}
      <View style={styles.actionButtonsContainer}>
        <TouchableOpacity
          style={[styles.actionButton, { backgroundColor: theme.primary }]}
          onPress={() => setShowJoinModal(true)}
        >
          <Ionicons name="enter-outline" size={24} color="#fff" />
          <Text style={styles.actionButtonText}>Unirse</Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.actionButton, { backgroundColor: theme.primary }]}
          onPress={() => {
            setEditingGroup(null)
            setShowForm(true)
          }}
        >
          <Ionicons name="add" size={24} color="#fff" />
          <Text style={styles.actionButtonText}>Crear</Text>
        </TouchableOpacity>
      </View>

      {/* Modal de formulario */}
      <Modal
        visible={showForm}
        transparent={true}
        animationType="slide"
        onRequestClose={() => {
          setShowForm(false)
          setEditingGroup(null)
        }}
      >
        <View style={styles.modalContainer}>
          <View style={[styles.modalContent, { backgroundColor: theme.background }]}>
            <EntityForm
              type="group"
              initialData={editingGroup || {}}
              onSubmit={(formData) => {
                if (editingGroup && editingGroup.id) {
                  handleUpdateGroup(formData)
                } else {
                  handleCreateGroup(formData)
                }
              }}
              onCancel={() => {
                setShowForm(false)
                setEditingGroup(null)
              }}
            />
          </View>
        </View>
      </Modal>

      {/* Modal para unirse a grupo */}
      <Modal
        visible={showJoinModal}
        transparent={true}
        animationType="slide"
        onRequestClose={() => setShowJoinModal(false)}
      >
        <View style={styles.modalContainer}>
          <View style={[styles.joinModalContent, { backgroundColor: theme.background }]}>
            <Text style={[styles.joinModalTitle, { color: theme.text }]}>Unirse a un Grupo</Text>

            <Text style={[styles.joinModalSubtitle, { color: theme.text + "CC" }]}>
              Ingrese el código de invitación
            </Text>

            <TextInput
              style={[
                styles.joinInput,
                {
                  backgroundColor: theme.card,
                  color: theme.text,
                  borderColor: theme.border,
                },
              ]}
              placeholder="Código de invitación"
              placeholderTextColor={theme.text + "80"}
              value={joinCode}
              onChangeText={setJoinCode}
              autoCapitalize="characters"
            />

            <View style={styles.joinButtonsContainer}>
              <TouchableOpacity
                style={[styles.joinButton, styles.cancelJoinButton, { borderColor: theme.border }]}
                onPress={() => {
                  setShowJoinModal(false)
                  setJoinCode("")
                }}
              >
                <Text style={[styles.joinButtonText, { color: theme.text }]}>Cancelar</Text>
              </TouchableOpacity>

              <TouchableOpacity
                style={[styles.joinButton, styles.confirmJoinButton, { backgroundColor: theme.primary }]}
                onPress={handleJoinGroup}
              >
                <Text style={[styles.joinButtonText, { color: "#fff" }]}>Unirse</Text>
              </TouchableOpacity>
            </View>
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
  listContainer: {
    padding: 15,
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
  actionButtonsContainer: {
    position: "absolute",
    bottom: 20,
    right: 20,
    flexDirection: "row",
  },
  actionButton: {
    width: 56,
    height: 56,
    borderRadius: 28,
    justifyContent: "center",
    alignItems: "center",
    marginLeft: 10,
    elevation: 5,
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 3.84,
  },
  actionButtonText: {
    color: "#fff",
    fontSize: 12,
    marginTop: 2,
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
  joinModalContent: {
    margin: 20,
    borderRadius: 10,
    padding: 20,
    elevation: 5,
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 3.84,
  },
  joinModalTitle: {
    fontSize: 24,
    fontWeight: "bold",
    marginBottom: 10,
    textAlign: "center",
  },
  joinModalSubtitle: {
    fontSize: 16,
    marginBottom: 20,
    textAlign: "center",
  },
  joinInput: {
    height: 50,
    borderWidth: 1,
    borderRadius: 8,
    paddingHorizontal: 15,
    fontSize: 16,
    marginBottom: 20,
  },
  joinButtonsContainer: {
    flexDirection: "row",
    justifyContent: "space-between",
  },
  joinButton: {
    flex: 1,
    height: 50,
    borderRadius: 8,
    justifyContent: "center",
    alignItems: "center",
  },
  cancelJoinButton: {
    marginRight: 10,
    borderWidth: 1,
  },
  confirmJoinButton: {
    marginLeft: 10,
  },
  joinButtonText: {
    fontSize: 16,
    fontWeight: "bold",
  },
})

export default GroupsScreen
