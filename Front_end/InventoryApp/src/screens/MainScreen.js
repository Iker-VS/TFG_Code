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
import BreadcrumbNavigation from "../components/BreadcrumbNavigation"
import EntityPanel from "../components/EntityPanel"
import SwipeablePanel from "../components/SwipeablePanel"
import EntityForm from "../components/EntityForm"
import {
  fetchGroups,
  fetchProperties,
  fetchZones,
  fetchSubZones,
  fetchItems,
  createProperty,
  createZone,
  createItem,
  updateProperty,
  updateZone,
  updateItem,
  deleteProperty,
  deleteZone,
  deleteItem,
} from "../services/api"

const MainScreen = ({ navigation }) => {
  const { theme } = useContext(ThemeContext)
  const { userToken } = useContext(AuthContext)

  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState(null)
  const [entities, setEntities] = useState([])
  const [navigationPath, setNavigationPath] = useState([])
  const [currentLevel, setCurrentLevel] = useState("group")
  const [currentParentId, setCurrentParentId] = useState(null)
  const [searchQuery, setSearchQuery] = useState("")
  const [showForm, setShowForm] = useState(false)
  const [editingEntity, setEditingEntity] = useState(null)
  const [showAddButtons, setShowAddButtons] = useState(false)

  // Cargar datos iniciales al enfocar la pantalla
  useFocusEffect(
    useCallback(() => {
      loadData()
    }, [userToken]),
  )

  // Cargar datos según el nivel actual
  const loadData = async () => {
    if (!userToken) return

    setIsLoading(true)
    setError(null)

    try {
      let data = []

      if (navigationPath.length === 0) {
        // Nivel de grupos
        const response = await fetchGroups()
        data = response
        setCurrentLevel("group")
        setCurrentParentId(null)
      } else {
        const lastItem = navigationPath[navigationPath.length - 1]

        if (lastItem.type === "group") {
          // Nivel de propiedades
          const response = await fetchProperties(lastItem.id)
          data = response
          setCurrentLevel("property")
          setCurrentParentId(lastItem.id)
        } else if (lastItem.type === "property") {
          // Nivel de zonas (de una propiedad)
          const response = await fetchZones(lastItem.id)
          data = response
          setCurrentLevel("zone")
          setCurrentParentId(lastItem.id)
        } else if (lastItem.type === "zone") {
          // Nivel de subzonas y objetos
          const zonesResponse = await fetchSubZones(lastItem.id)
          const itemsResponse = await fetchItems(lastItem.id)

          // Marcar las zonas y objetos para diferenciarlos
          const zones = zonesResponse.map((zone) => ({ ...zone, entityType: "zone" }))
          const items = itemsResponse.map((item) => ({ ...item, entityType: "item" }))

          data = [...zones, ...items]
          setCurrentLevel("zone")
          setCurrentParentId(lastItem.id)
        }
      }

      setEntities(data)
    } catch (err) {
      console.error("Error loading data:", err)
      setError("Error al cargar los datos. Intente nuevamente.")
    } finally {
      setIsLoading(false)
    }
  }

  // Navegar a un nivel específico
  const navigateTo = (entity) => {
    const newPath = [
      ...navigationPath,
      {
        id: entity.id,
        name: entity.name,
        type: entity.entityType || currentLevel,
      },
    ]

    setNavigationPath(newPath)
  }

  // Navegar hacia atrás en la ruta
  const navigateBack = (index) => {
    const newPath = navigationPath.slice(0, index + 1)
    setNavigationPath(newPath)
  }

  // Crear una nueva entidad
  const handleCreate = async (formData) => {
    setIsLoading(true)
    setError(null)

    try {
      let response

      if (currentLevel === "group") {
        // No se pueden crear grupos desde aquí, se hace desde la pantalla de grupos
      } else if (currentLevel === "property") {
        response = await createProperty(currentParentId, formData)
      } else if (currentLevel === "zone") {
        if (formData.entityType === "item") {
          response = await createItem(currentParentId, formData)
        } else {
          const isProperty = navigationPath[navigationPath.length - 1].type === "property"
          response = await createZone(currentParentId, isProperty, formData)
        }
      }

      setShowForm(false)
      setEditingEntity(null)
      loadData()
    } catch (err) {
      console.error("Error creating entity:", err)
      setError("Error al crear. Intente nuevamente.")
    } finally {
      setIsLoading(false)
    }
  }

  // Actualizar una entidad existente
  const handleUpdate = async (formData) => {
    setIsLoading(true)
    setError(null)

    try {
      if (editingEntity.entityType === "property" || currentLevel === "property") {
        await updateProperty(editingEntity.id, formData)
      } else if (editingEntity.entityType === "zone" || (currentLevel === "zone" && !editingEntity.entityType)) {
        await updateZone(editingEntity.id, formData)
      } else if (editingEntity.entityType === "item") {
        await updateItem(editingEntity.id, formData)
      }

      setShowForm(false)
      setEditingEntity(null)
      loadData()
    } catch (err) {
      console.error("Error updating entity:", err)
      setError("Error al actualizar. Intente nuevamente.")
    } finally {
      setIsLoading(false)
    }
  }

  // Eliminar una entidad
  const handleDelete = (entity) => {
    Alert.alert("Confirmar eliminación", `¿Está seguro que desea eliminar ${entity.name}?`, [
      { text: "No", style: "cancel" },
      {
        text: "Sí",
        style: "destructive",
        onPress: async () => {
          setIsLoading(true)
          setError(null)

          try {
            if (entity.entityType === "property" || currentLevel === "property") {
              await deleteProperty(entity.id)
            } else if (entity.entityType === "zone" || (currentLevel === "zone" && !entity.entityType)) {
              await deleteZone(entity.id)
            } else if (entity.entityType === "item") {
              await deleteItem(entity.id)
            }

            loadData()
          } catch (err) {
            console.error("Error deleting entity:", err)
            setError("Error al eliminar. Intente nuevamente.")
          } finally {
            setIsLoading(false)
          }
        },
      },
    ])
  }

  // Renderizar el botón de agregar según el nivel actual
  const renderAddButton = () => {
    if (currentLevel === "group") {
      return null // No se pueden crear grupos desde aquí
    }

    if (currentLevel === "zone" && navigationPath[navigationPath.length - 1].type === "zone") {
      // En una zona, mostrar botón que despliega opciones para agregar zona u objeto
      return (
        <View style={styles.addButtonContainer}>
          {showAddButtons ? (
            <View style={styles.addButtonsExpanded}>
              <TouchableOpacity
                style={[styles.addButton, { backgroundColor: theme.primary }]}
                onPress={() => {
                  setEditingEntity({ entityType: "zone" })
                  setShowForm(true)
                  setShowAddButtons(false)
                }}
              >
                <Ionicons name="grid-outline" size={24} color="#fff" />
                <Text style={styles.addButtonText}>Agregar Zona</Text>
              </TouchableOpacity>

              <TouchableOpacity
                style={[styles.addButton, { backgroundColor: theme.primary }]}
                onPress={() => {
                  setEditingEntity({ entityType: "item" })
                  setShowForm(true)
                  setShowAddButtons(false)
                }}
              >
                <Ionicons name="cube-outline" size={24} color="#fff" />
                <Text style={styles.addButtonText}>Agregar Objeto</Text>
              </TouchableOpacity>

              <TouchableOpacity
                style={[styles.closeButton, { backgroundColor: theme.error }]}
                onPress={() => setShowAddButtons(false)}
              >
                <Ionicons name="close" size={24} color="#fff" />
              </TouchableOpacity>
            </View>
          ) : (
            <TouchableOpacity
              style={[styles.addButton, { backgroundColor: theme.primary }]}
              onPress={() => setShowAddButtons(true)}
            >
              <Ionicons name="add" size={24} color="#fff" />
            </TouchableOpacity>
          )}
        </View>
      )
    }

    // Para propiedad o zona de propiedad, mostrar un solo botón
    return (
      <View style={styles.addButtonContainer}>
        <TouchableOpacity
          style={[styles.addButton, { backgroundColor: theme.primary }]}
          onPress={() => {
            setEditingEntity(null)
            setShowForm(true)
          }}
        >
          <Ionicons name="add" size={24} color="#fff" />
        </TouchableOpacity>
      </View>
    )
  }

  // Determinar el tipo de entidad para el formulario
  const getFormEntityType = () => {
    if (editingEntity) {
      return editingEntity.entityType || currentLevel
    }

    return currentLevel
  }

  return (
    <View style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Barra de búsqueda */}
      <View style={[styles.searchContainer, { backgroundColor: theme.card, borderColor: theme.border }]}>
        <Ionicons name="search" size={20} color={theme.text + "80"} />
        <TextInput
          style={[styles.searchInput, { color: theme.text }]}
          placeholder="Buscar..."
          placeholderTextColor={theme.text + "80"}
          value={searchQuery}
          onChangeText={setSearchQuery}
        />
      </View>

      {/* Navegación de ruta */}
      <BreadcrumbNavigation
        path={[{ id: "root", name: "Inicio", type: "root" }, ...navigationPath]}
        onNavigate={(index) => {
          if (index === 0) {
            setNavigationPath([])
          } else {
            navigateBack(index - 1)
          }
        }}
      />

      {/* Lista de entidades */}
      {isLoading ? (
        <View style={styles.centerContainer}>
          <ActivityIndicator size="large" color={theme.primary} />
        </View>
      ) : error ? (
        <View style={styles.centerContainer}>
          <Text style={[styles.errorText, { color: theme.error }]}>{error}</Text>
          <TouchableOpacity style={[styles.retryButton, { backgroundColor: theme.primary }]} onPress={loadData}>
            <Text style={styles.retryButtonText}>Reintentar</Text>
          </TouchableOpacity>
        </View>
      ) : entities.length === 0 ? (
        <View style={styles.centerContainer}>
          <Text style={[styles.emptyText, { color: theme.text + "CC" }]}>No hay elementos para mostrar</Text>
        </View>
      ) : (
        <FlatList
          data={entities}
          keyExtractor={(item) => item.id.toString()}
          contentContainerStyle={styles.listContainer}
          renderItem={({ item }) => (
            <SwipeablePanel
              onEdit={() => {
                setEditingEntity(item)
                setShowForm(true)
              }}
              onDelete={() => handleDelete(item)}
            >
              <EntityPanel
                entity={item}
                type={item.entityType || currentLevel}
                onPress={() => {
                  if (item.entityType === "item") {
                    // Los objetos no tienen subniveles, mostrar detalles o algo similar
                    Alert.alert("Detalles del objeto", JSON.stringify(item, null, 2))
                  } else {
                    navigateTo(item)
                  }
                }}
              />
            </SwipeablePanel>
          )}
        />
      )}

      {/* Botón de agregar */}
      {renderAddButton()}

      {/* Modal de formulario */}
      <Modal
        visible={showForm}
        transparent={true}
        animationType="slide"
        onRequestClose={() => {
          setShowForm(false)
          setEditingEntity(null)
        }}
      >
        <View style={styles.modalContainer}>
          <View style={[styles.modalContent, { backgroundColor: theme.background }]}>
            <EntityForm
              type={getFormEntityType()}
              initialData={editingEntity || {}}
              onSubmit={(formData) => {
                if (editingEntity && editingEntity.id) {
                  handleUpdate(formData)
                } else {
                  handleCreate(formData)
                }
              }}
              onCancel={() => {
                setShowForm(false)
                setEditingEntity(null)
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
  searchContainer: {
    flexDirection: "row",
    alignItems: "center",
    margin: 10,
    paddingHorizontal: 15,
    height: 40,
    borderRadius: 20,
    borderWidth: 1,
  },
  searchInput: {
    flex: 1,
    marginLeft: 10,
    fontSize: 16,
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
  addButtonContainer: {
    position: "absolute",
    bottom: 20,
    right: 20,
  },
  addButton: {
    width: 56,
    height: 56,
    borderRadius: 28,
    justifyContent: "center",
    alignItems: "center",
    elevation: 5,
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 3.84,
  },
  addButtonsExpanded: {
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
  },
  addButtonText: {
    color: "#fff",
    fontSize: 12,
    marginTop: 5,
  },
  closeButton: {
    width: 56,
    height: 56,
    borderRadius: 28,
    justifyContent: "center",
    alignItems: "center",
    marginTop: 10,
    elevation: 5,
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 3.84,
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

export default MainScreen
