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
  Switch,
} from "react-native"
import { useFocusEffect } from "@react-navigation/native"
import { Ionicons } from "@expo/vector-icons"
import { ThemeContext } from "../context/ThemeContext"
import { AuthContext } from "../context/AuthContext"
import EntityPanel from "../components/EntityPanel"
import SwipeablePanel from "../components/SwipeablePanel"
import {
  fetchUserGroups,
  checkGroupByCode,
  joinGroup,
  updateGroup,
  leaveGroup,
  createGroup,
  checkUserInGroup,
} from "../services/api"
import { normalizeId, isValidId, retryOperation } from "../utils/idUtils"

const GroupsScreen = ({ navigation }) => {
  const { theme } = useContext(ThemeContext)
  const { userToken, userData } = useContext(AuthContext)

  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState(null)
  const [groups, setGroups] = useState([])
  const [showForm, setShowForm] = useState(false)
  const [editingGroup, setEditingGroup] = useState(null)
  const [showJoinModal, setShowJoinModal] = useState(false)
  const [joinCode, setJoinCode] = useState("")

  // Campos para crear/editar grupo
  const [groupName, setGroupName] = useState("")
  const [groupDescription, setGroupDescription] = useState("")
  const [hasUserMax, setHasUserMax] = useState(false)
  const [userMax, setUserMax] = useState("10")

  // Cargar datos iniciales al enfocar la pantalla
  useFocusEffect(
    useCallback(() => {
      if (userToken && userData) {
        loadGroups()
      }
    }, [userToken, userData]),
  )

  // Cargar grupos del usuario
  const loadGroups = async () => {
    if (!userToken || !userData || !userData._id) return

    setIsLoading(true)
    setError(null)

    try {
      // Normalizar el ID del usuario
      const userId = normalizeId(userData._id)

      if (!isValidId(userId)) {
        console.error("Invalid user ID format:", userData._id)
        setGroups([])
        setIsLoading(false)
        return
      }

      // Usar la nueva función para obtener los grupos del usuario
      const response = await fetchUserGroups(userId)
      setGroups(response || []) // Ensure we always have an array
    } catch (err) {
      console.error("Error loading groups:", err)
      // Don't set error, just show empty state
      setGroups([])
    } finally {
      setIsLoading(false)
    }
  }

  // Función segura para extraer el ID
  const getGroupId = (item) => {
    return normalizeId(item?._id) || Math.random().toString()
  }

  // Preparar el formulario para crear un grupo
  const prepareCreateGroup = () => {
    setGroupName("")
    setGroupDescription("")
    setHasUserMax(false)
    setUserMax("10")
    setEditingGroup(null)
    setShowForm(true)
  }

  // Preparar el formulario para editar un grupo
  const prepareEditGroup = (group) => {
    setGroupName(group.name || "")
    setGroupDescription(group.description || "")
    setHasUserMax(group.userMax !== null && group.userMax !== undefined)
    setUserMax(group.userMax ? group.userMax.toString() : "10")
    setEditingGroup(group)
    setShowForm(true)
  }

  // Crear un nuevo grupo
  const handleCreateGroup = async () => {
    if (!groupName.trim()) {
      Alert.alert("Error", "Por favor ingrese un nombre para el grupo")
      return
    }

    setIsLoading(true)
    setError(null)

    try {
      const groupData = {
        name: groupName,
        description: groupDescription,
        userMax: hasUserMax ? Number.parseInt(userMax) : null,
      }

      // 1. Create the group
      const newGroup = await createGroup(groupData)
      console.log("Group created successfully:", newGroup)

      // Verificar y extraer el ID del grupo correctamente
      const groupId = normalizeId(newGroup._id)

      if (!isValidId(normalizeId(newGroup._id))) {
        throw new Error("Failed to create group: No valid group ID returned")
      }

      // 2. Normalizar el ID del usuario
      const userId = normalizeId(userData._id)

      if (!isValidId(userId)) {
        throw new Error("Invalid user ID format")
      }

      // 3. Añadir el usuario al grupo con reintentos
      await retryOperation(
        async () => {
          await joinGroup(groupId, userId)
          console.log("User added to group successfully")

          // Verificar que el usuario se haya añadido correctamente
          const isUserInGroup = await checkUserInGroup(groupId, userId)
          if (!isUserInGroup) {
            throw new Error("Failed to verify user-group relationship")
          }
        },
        3,
        500,
      )

      // 4. Actualizar el contador de usuarios del grupo
      const updatedGroup = {
        ...newGroup,
        _id: groupId, // Asegurar que el ID está normalizado
        userCount: 1, // Set to 1 since this is the first user
      }

      await updateGroup(groupId, updatedGroup)
      console.log("Group user count updated successfully")

      setShowForm(false)

      // 5. Reload groups to show the new group
      await loadGroups()

      // 6. Navigate to the Home screen with the new group
      navigation.navigate("Home", { group: updatedGroup })

      Alert.alert("Success", "Group created and joined successfully")
    } catch (err) {
      console.error("Error creating group:", err)
      setError("Error al crear el grupo. Intente nuevamente.")
      Alert.alert("Error", "Failed to create group: " + (err.message || "Unknown error"))
    } finally {
      setIsLoading(false)
    }
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
      // 1. Verificar si el grupo existe
      const group = await checkGroupByCode(joinCode)

      if (!group) {
        Alert.alert("Error", "El código ingresado no corresponde a ningún grupo")
        setIsLoading(false)
        return
      }

      // 2. Verificar si el grupo está lleno
      if (group.userMax !== null && group.userCount >= group.userMax) {
        Alert.alert("Error", "El grupo ha alcanzado su capacidad máxima")
        setIsLoading(false)
        return
      }

      // 3. Normalizar el ID del usuario
      const userId = normalizeId(userData._id)

      if (!isValidId(userId)) {
        throw new Error("Invalid user ID format")
      }

      // 4. Normalizar el ID del grupo
      const groupId = normalizeId(group._id)

      if (!isValidId(groupId)) {
        throw new Error("Invalid group ID format")
      }

      // 5. Crear relación usuario-grupo con reintentos
      await retryOperation(
        async () => {
          await joinGroup(groupId, userId)

          // Verificar que el usuario se haya añadido correctamente
          const isUserInGroup = await checkUserInGroup(groupId, userId)
          if (!isUserInGroup) {
            throw new Error("Failed to verify user-group relationship")
          }
        },
        3,
        500,
      )

      // 6. Esperar un momento para asegurar que la relación se ha creado
      await new Promise((resolve) => setTimeout(resolve, 500))

      // 7. Actualizar el contador de usuarios del grupo
      const updatedGroup = {
        ...group,
        _id: groupId, // Asegurar que el ID está normalizado
        userCount: group.userCount + 1,
      }

      await updateGroup(groupId, updatedGroup)

      setShowJoinModal(false)
      setJoinCode("")

      // 8. Reload groups
      await loadGroups()

      // 9. Navigate to the Home screen with the joined group
      navigation.navigate("Home", { group: updatedGroup })

      Alert.alert("Éxito", "Te has unido al grupo correctamente")
    } catch (err) {
      console.error("Error joining group:", err)
      Alert.alert("Error", "No se pudo unir al grupo: " + (err.message || "Error desconocido"))
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
            // 1. Normalizar el ID del usuario
            const userId = normalizeId(userData._id)

            if (!isValidId(userId)) {
              throw new Error("Invalid user ID format")
            }

            // 2. Normalizar el ID del grupo
            const groupId = normalizeId(group._id)

            if (!isValidId(groupId)) {
              throw new Error("Invalid group ID format")
            }

            // 3. Eliminar relación usuario-grupo
            await leaveGroup(groupId, userId)

            // 4. Actualizar el contador de usuarios del grupo
            const updatedGroup = {
              ...group,
              _id: groupId, // Asegurar que el ID está normalizado
              userCount: Math.max(0, group.userCount - 1),
            }

            await updateGroup(groupId, updatedGroup)

            loadGroups()
          } catch (err) {
            console.error("Error leaving group:", err)
            setError("Error al salir del grupo. Intente nuevamente.")
            Alert.alert("Error", "No se pudo salir del grupo: " + (err.message || "Error desconocido"))
          } finally {
            setIsLoading(false)
          }
        },
      },
    ])
  }

  // Renderizar mensaje cuando no hay grupos
  const renderEmptyGroups = () => (
    <View style={styles.emptyContainer}>
      <Ionicons name="people-outline" size={80} color={theme.text + "80"} />
      <Text style={[styles.emptyTitle, { color: theme.text }]}>No groups found</Text>
      <Text style={[styles.emptyText, { color: theme.text + "CC" }]}>Join or create one to get started!</Text>
      <View style={styles.emptyButtonsContainer}>
        <TouchableOpacity
          style={[styles.emptyButton, { backgroundColor: theme.primary }]}
          onPress={() => setShowJoinModal(true)}
        >
          <Ionicons name="enter-outline" size={20} color="#fff" />
          <Text style={styles.emptyButtonText}>Join a group</Text>
        </TouchableOpacity>
        <TouchableOpacity style={[styles.emptyButton, { backgroundColor: theme.primary }]} onPress={prepareCreateGroup}>
          <Ionicons name="add" size={20} color="#fff" />
          <Text style={styles.emptyButtonText}>Create a group</Text>
        </TouchableOpacity>
      </View>
    </View>
  )

  return (
    <View style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Lista de grupos */}
      {isLoading ? (
        <View style={styles.centerContainer}>
          <ActivityIndicator size="large" color={theme.primary} />
        </View>
      ) : groups.length === 0 ? (
        renderEmptyGroups()
      ) : (
        <FlatList
          data={groups}
          keyExtractor={(item) => getGroupId(item)}
          contentContainerStyle={styles.listContainer}
          renderItem={({ item }) => (
            <SwipeablePanel onEdit={() => prepareEditGroup(item)} onDelete={() => handleLeaveGroup(item)}>
              <EntityPanel
                entity={{
                  name: item.name || "Sin nombre",
                  description: item.description || "Sin descripción",
                  userCount: item.userCount || 0,
                  userMax: item.userMax,
                }}
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

      {/* Always show action buttons, regardless of whether there are groups */}
      {!isLoading && (
        <View style={styles.actionButtonsContainer}>
          <TouchableOpacity
            style={[styles.actionButton, { backgroundColor: theme.primary }]}
            onPress={() => setShowJoinModal(true)}
          >
            <Ionicons name="enter-outline" size={24} color="#fff" />
            <Text style={styles.actionButtonText}>Join</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.actionButton, { backgroundColor: theme.primary }]}
            onPress={prepareCreateGroup}
          >
            <Ionicons name="add" size={24} color="#fff" />
            <Text style={styles.actionButtonText}>Create</Text>
          </TouchableOpacity>
        </View>
      )}

      {/* Modal para crear/editar grupo */}
      <Modal visible={showForm} transparent={true} animationType="slide" onRequestClose={() => setShowForm(false)}>
        <View style={styles.modalContainer}>
          <View style={[styles.formModalContent, { backgroundColor: theme.background }]}>
            <Text style={[styles.modalTitle, { color: theme.text }]}>
              {editingGroup ? "Editar Grupo" : "Crear Grupo"}
            </Text>

            <View style={styles.formField}>
              <Text style={[styles.formLabel, { color: theme.text }]}>Nombre del grupo *</Text>
              <TextInput
                style={[
                  styles.formInput,
                  {
                    backgroundColor: theme.card,
                    color: theme.text,
                    borderColor: theme.border,
                  },
                ]}
                placeholder="Nombre del grupo"
                placeholderTextColor={theme.text + "80"}
                value={groupName}
                onChangeText={setGroupName}
              />
            </View>

            <View style={styles.formField}>
              <Text style={[styles.formLabel, { color: theme.text }]}>Descripción</Text>
              <TextInput
                style={[
                  styles.formTextArea,
                  {
                    backgroundColor: theme.card,
                    color: theme.text,
                    borderColor: theme.border,
                  },
                ]}
                placeholder="Descripción del grupo (opcional)"
                placeholderTextColor={theme.text + "80"}
                value={groupDescription}
                onChangeText={setGroupDescription}
                multiline
                numberOfLines={4}
                textAlignVertical="top"
              />
            </View>

            <View style={styles.formField}>
              <View style={styles.switchContainer}>
                <Text style={[styles.formLabel, { color: theme.text }]}>Limitar número de usuarios</Text>
                <Switch
                  value={hasUserMax}
                  onValueChange={setHasUserMax}
                  trackColor={{ false: theme.border, true: theme.primary + "80" }}
                  thumbColor={hasUserMax ? theme.primary : theme.text + "40"}
                />
              </View>

              {hasUserMax && (
                <TextInput
                  style={[
                    styles.formInput,
                    {
                      backgroundColor: theme.card,
                      color: theme.text,
                      borderColor: theme.border,
                    },
                  ]}
                  placeholder="Número máximo de usuarios"
                  placeholderTextColor={theme.text + "80"}
                  value={userMax}
                  onChangeText={setUserMax}
                  keyboardType="numeric"
                />
              )}
            </View>

            <View style={styles.formButtons}>
              <TouchableOpacity
                style={[styles.formButton, styles.cancelButton, { borderColor: theme.border }]}
                onPress={() => setShowForm(false)}
              >
                <Text style={[styles.formButtonText, { color: theme.text }]}>Cancelar</Text>
              </TouchableOpacity>

              <TouchableOpacity
                style={[styles.formButton, styles.submitButton, { backgroundColor: theme.primary }]}
                onPress={handleCreateGroup}
              >
                <Text style={[styles.formButtonText, { color: "#fff" }]}>{editingGroup ? "Guardar" : "Crear"}</Text>
              </TouchableOpacity>
            </View>
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
  formModalContent: {
    margin: 20,
    borderRadius: 10,
    padding: 20,
    elevation: 5,
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 3.84,
  },
  modalTitle: {
    fontSize: 24,
    fontWeight: "bold",
    marginBottom: 20,
    textAlign: "center",
  },
  formField: {
    marginBottom: 15,
  },
  formLabel: {
    fontSize: 16,
    marginBottom: 5,
  },
  formInput: {
    height: 50,
    borderWidth: 1,
    borderRadius: 8,
    paddingHorizontal: 15,
    fontSize: 16,
  },
  formTextArea: {
    minHeight: 100,
    borderWidth: 1,
    borderRadius: 8,
    paddingHorizontal: 15,
    paddingVertical: 10,
    fontSize: 16,
  },
  switchContainer: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: 10,
  },
  formButtons: {
    flexDirection: "row",
    justifyContent: "space-between",
    marginTop: 20,
  },
  formButton: {
    flex: 1,
    height: 50,
    borderRadius: 8,
    justifyContent: "center",
    alignItems: "center",
  },
  cancelButton: {
    marginRight: 10,
    borderWidth: 1,
  },
  submitButton: {
    marginLeft: 10,
  },
  formButtonText: {
    fontSize: 16,
    fontWeight: "bold",
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
  emptyContainer: {
    flex: 1,
    justifyContent: "center",
    alignItems: "center",
    padding: 30,
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: "bold",
    marginTop: 20,
    marginBottom: 10,
    textAlign: "center",
  },
  emptyText: {
    fontSize: 16,
    textAlign: "center",
    marginBottom: 30,
  },
  emptyButtonsContainer: {
    width: "100%",
    maxWidth: 300,
  },
  emptyButton: {
    flexDirection: "row",
    height: 50,
    borderRadius: 8,
    justifyContent: "center",
    alignItems: "center",
    marginBottom: 15,
  },
  emptyButtonText: {
    color: "#fff",
    fontSize: 16,
    fontWeight: "bold",
    marginLeft: 10,
  },
})

export default GroupsScreen
