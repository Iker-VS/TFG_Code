"use client"

import { useContext } from "react"
import { View, Text, StyleSheet, TouchableOpacity } from "react-native"
import { ThemeContext } from "../context/ThemeContext"
import { Ionicons } from "@expo/vector-icons"

const EntityPanel = ({ entity, onPress, type }) => {
  const { theme } = useContext(ThemeContext)

  // Determinar qué campos mostrar según el tipo de entidad
  const renderFields = () => {
    switch (type) {
      case "group":
        return (
          <>
            <Text style={[styles.entityName, { color: theme.text }]}>{entity.name}</Text>
            <Text style={[styles.entityDetail, { color: theme.text + "CC" }]}>
              {entity.description || "Sin descripción"}
            </Text>
          </>
        )
      case "property":
        return (
          <>
            <Text style={[styles.entityName, { color: theme.text }]}>{entity.name}</Text>
            <Text style={[styles.entityDetail, { color: theme.text + "CC" }]}>{entity.address || "Sin dirección"}</Text>
          </>
        )
      case "zone":
        return (
          <>
            <Text style={[styles.entityName, { color: theme.text }]}>{entity.name}</Text>
            <Text style={[styles.entityDetail, { color: theme.text + "CC" }]}>
              {entity.description || "Sin descripción"}
            </Text>
          </>
        )
      case "item":
        return (
          <>
            <Text style={[styles.entityName, { color: theme.text }]}>{entity.name}</Text>
            <Text style={[styles.entityDetail, { color: theme.text + "CC" }]}>
              {entity.description || "Sin descripción"}
            </Text>
            <Text style={[styles.entityDetail, { color: theme.text + "CC" }]}>
              Estado: {entity.status || "No especificado"}
            </Text>
          </>
        )
      default:
        return <Text style={[styles.entityName, { color: theme.text }]}>{entity.name}</Text>
    }
  }

  return (
    <TouchableOpacity
      style={[styles.container, { backgroundColor: theme.panel, borderColor: theme.border }]}
      onPress={onPress}
    >
      <View style={styles.content}>
        <View style={[styles.imageContainer, { backgroundColor: theme.border + "50" }]}>
          <Ionicons
            name={type === "group" ? "people" : type === "property" ? "home" : type === "zone" ? "grid" : "cube"}
            size={24}
            color={theme.primary}
          />
        </View>
        <View style={styles.infoContainer}>{renderFields()}</View>
      </View>
    </TouchableOpacity>
  )
}

const styles = StyleSheet.create({
  container: {
    borderRadius: 8,
    borderWidth: 1,
    marginBottom: 10,
    overflow: "hidden",
  },
  content: {
    flexDirection: "row",
    padding: 15,
  },
  imageContainer: {
    width: 60,
    height: 60,
    borderRadius: 8,
    justifyContent: "center",
    alignItems: "center",
    marginRight: 15,
  },
  infoContainer: {
    flex: 1,
    justifyContent: "center",
  },
  entityName: {
    fontSize: 16,
    fontWeight: "bold",
    marginBottom: 4,
  },
  entityDetail: {
    fontSize: 14,
    marginBottom: 2,
  },
})

export default EntityPanel
