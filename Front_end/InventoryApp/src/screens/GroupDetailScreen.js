"use client"

import { useState, useContext, useCallback } from "react"
import { View, Text, StyleSheet, FlatList, TouchableOpacity, Alert, ActivityIndicator } from "react-native"
import { useFocusEffect } from "@react-navigation/native"
import { Ionicons } from "@expo/vector-icons"
import { ThemeContext } from "../context/ThemeContext"
import { AuthContext } from "../context/AuthContext"
import BreadcrumbNavigation from "../components/BreadcrumbNavigation"
import EntityPanel from "../components/EntityPanel"
import SwipeablePanel from "../components/SwipeablePanel"
import { fetchProperties } from "../services/api"
import { normalizeId, isValidId } from "../utils/idUtils"

const GroupDetailScreen = ({ route, navigation }) => {
  const { theme } = useContext(ThemeContext)
  const { userToken } = useContext(AuthContext)
  const { group } = route.params || {}

  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState(null)
  const [properties, setProperties] = useState([])
  const [navigationPath, setNavigationPath] = useState([
    { id: "groups", name: "Mis Grupos", type: "root" },
    { id: normalizeId(group?._id), name: group?.name || "Grupo", type: "group" },
  ])
  const [currentEntity, setCurrentEntity] = useState(null)

  // Cargar datos iniciales al enfocar la pantalla
  useFocusEffect(
    useCallback(() => {
      if (userToken && group && group._id) {
        loadProperties()
      }
    }, [userToken, group]),
  )

  // Cargar propiedades del grupo
  const loadProperties = async () => {
    if (!userToken || !group || !group._id) return

    setIsLoading(true)
    setError(null)

    try {
      // Normalizar el ID del grupo
      const groupId = normalizeId(group._id)

      if (!isValidId(groupId)) {
        console.error("Invalid group ID format:", group._id)
        setProperties([])
        setIsLoading(false)
        return
      }

      // Cargar propiedades del grupo
      const response = await fetchProperties(groupId)
      setProperties(response || []) // Ensure we always have an array
    } catch (err) {
      console.error("Error loading properties:", err)
      setError("Error al cargar las propiedades. Intente nuevamente.")
      setProperties([])
    } finally {
      setIsLoading(false)
    }
  }

  // Función segura para extraer el ID
  const getEntityId = (item) => {
    return normalizeId(item?._id) || Math.random().toString()
  }

  // Navegar a una propiedad
  const navigateToProperty = (property) => {
    // Aquí implementarías la navegación a la pantalla de detalle de la propiedad
    Alert.alert("Propiedad seleccionada", `Has seleccionado la propiedad: ${property.name}`)
  }

  // Crear una nueva propiedad
  const handleCreateProperty = () => {
    // Aquí implementarías la lógica para crear una nueva propiedad
    Alert.alert("Crear propiedad", "Aquí se implementaría la creación de una nueva propiedad")
  }

  // Renderizar mensaje cuando no hay propiedades
  const renderEmptyProperties = () => (
    <View style={styles.emptyContainer}>
      <Ionicons name="home-outline" size={80} color={theme.text + "80"} />
      <Text style={[styles.emptyTitle, { color: theme.text }]}>No properties found</Text>
      <Text style={[styles.emptyText, { color: theme.text + "CC" }]}>
        Create a property to start organizing your inventory
      </Text>
      <TouchableOpacity style={[styles.emptyButton, { backgroundColor: theme.primary }]} onPress={handleCreateProperty}>
        <Ionicons name="add" size={20} color="#fff" />
        <Text style={styles.emptyButtonText}>Create Property</Text>
      </TouchableOpacity>
    </View>
  )

  // Renderizar información del grupo
  const renderGroupInfo = () => (
    <View style={[styles.groupInfoContainer, { backgroundColor: theme.card, borderColor: theme.border }]}>
      <Text style={[styles.groupName, { color: theme.text }]}>{group?.name || "Grupo"}</Text>
      <View style={styles.groupDetailsRow}>
        <View style={styles.groupDetailItem}>
          <Ionicons name="people" size={20} color={theme.primary} style={styles.groupDetailIcon} />
          <Text style={[styles.groupDetailText, { color: theme.text }]}>
            {group?.userCount || 0} {group?.userMax ? `/ ${group.userMax}` : ""} miembros
          </Text>
        </View>
        <View style={styles.groupDetailItem}>
          <Ionicons name="key" size={20} color={theme.primary} style={styles.groupDetailIcon} />
          <Text style={[styles.groupDetailText, { color: theme.text }]}>Código: {group?.groupCode || "N/A"}</Text>
        </View>
      </View>
    </View>
  )

  return (
    <View style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Navegación de ruta */}
      <BreadcrumbNavigation
        path={navigationPath}
        onNavigate={(index) => {
          if (index === 0) {
            // Navegar a la pantalla de grupos
            navigation.navigate("Groups")
          }
        }}
      />

      {/* Información del grupo */}
      {renderGroupInfo()}

      {/* Lista de propiedades */}
      {isLoading ? (
        <View style={styles.centerContainer}>
          <ActivityIndicator size="large" color={theme.primary} />
        </View>
      ) : error ? (
        <View style={styles.centerContainer}>
          <Text style={[styles.errorText, { color: theme.error }]}>{error}</Text>
          <TouchableOpacity style={[styles.retryButton, { backgroundColor: theme.primary }]} onPress={loadProperties}>
            <Text style={styles.retryButtonText}>Reintentar</Text>
          </TouchableOpacity>
        </View>
      ) : properties.length === 0 ? (
        renderEmptyProperties()
      ) : (
        <FlatList
          data={properties}
          keyExtractor={(item) => getEntityId(item)}
          contentContainerStyle={styles.listContainer}
          renderItem={({ item }) => (
            <SwipeablePanel
              onEdit={() => {
                // Aquí implementarías la edición de una propiedad
                Alert.alert("Editar propiedad", `Editar la propiedad: ${item.name}`)
              }}
              onDelete={() => {
                // Aquí implementarías la eliminación de una propiedad
                Alert.alert("Eliminar propiedad", `Eliminar la propiedad: ${item.name}`)
              }}
              leftActionLabel="Edit"
              rightActionLabel="Delete"
              immediateAction={true}
            >
              <EntityPanel
                entity={{
                  name: item.name || "Sin nombre",
                  address: item.direction || "Sin dirección",
                }}
                type="property"
                onPress={() => navigateToProperty(item)}
              />
            </SwipeablePanel>
          )}
        />
      )}

      {/* Botón para crear una nueva propiedad */}
      <TouchableOpacity style={[styles.addButton, { backgroundColor: theme.primary }]} onPress={handleCreateProperty}>
        <Ionicons name="add" size={24} color="#fff" />
      </TouchableOpacity>
    </View>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  groupInfoContainer: {
    padding: 15,
    borderRadius: 8,
    borderWidth: 1,
    margin: 15,
    marginTop: 5,
  },
  groupName: {
    fontSize: 20,
    fontWeight: "bold",
    marginBottom: 10,
  },
  groupDetailsRow: {
    flexDirection: "row",
    justifyContent: "space-between",
  },
  groupDetailItem: {
    flexDirection: "row",
    alignItems: "center",
  },
  groupDetailIcon: {
    marginRight: 5,
  },
  groupDetailText: {
    fontSize: 14,
  },
  listContainer: {
    padding: 15,
    paddingTop: 0,
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
  emptyButton: {
    flexDirection: "row",
    height: 50,
    borderRadius: 8,
    justifyContent: "center",
    alignItems: "center",
    paddingHorizontal: 20,
  },
  emptyButtonText: {
    color: "#fff",
    fontSize: 16,
    fontWeight: "bold",
    marginLeft: 10,
  },
  addButton: {
    position: "absolute",
    bottom: 20,
    right: 20,
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
})

export default GroupDetailScreen
