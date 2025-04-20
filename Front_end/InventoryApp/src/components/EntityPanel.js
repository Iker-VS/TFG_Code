"use client"

import { useContext } from "react"
import { View, Text, StyleSheet, TouchableOpacity } from "react-native"
import { ThemeContext } from "../context/ThemeContext"
import { Ionicons } from "@expo/vector-icons"

const EntityPanel = ({ entity, onPress, type }) => {
  const { theme } = useContext(ThemeContext)

  // Ensure entity has all required properties with defaults
  const safeEntity = {
    name: entity?.name || "Sin nombre",
    address: entity?.address || "",
    status: entity?.status || "",
    userCount: entity?.userCount || 0,
    userMax: entity?.userMax || null,
    groupCode: entity?.groupCode || "No disponible",
    ...entity,
  }

  // Enhance EntityPanel to display group information better
  const renderFields = () => {
    switch (type) {
      case "group":
        return (
          <>
            <Text style={[styles.entityName, { color: theme.text }]} numberOfLines={1} ellipsizeMode="tail">
              {safeEntity.name}
            </Text>
            <Text style={[styles.entityDetail, { color: theme.text + "CC" }]}>Código: {safeEntity.groupCode}</Text>
            {safeEntity.userCount !== undefined && (
              <Text style={[styles.entityDetail, { color: theme.text + "CC" }]}>
                Members: {safeEntity.userCount}
                {safeEntity.userMax ? ` / ${safeEntity.userMax}` : ""}
              </Text>
            )}
          </>
        )
      case "property":
        return (
          <>
            <Text style={[styles.entityName, { color: theme.text }]} numberOfLines={1} ellipsizeMode="tail">
              {safeEntity.name}
            </Text>
            <Text style={[styles.entityDetail, { color: theme.text + "CC" }]}>
              {safeEntity.address || "No address"}
            </Text>
          </>
        )
      case "zone":
        return (
          <>
            <Text style={[styles.entityName, { color: theme.text }]} numberOfLines={1} ellipsizeMode="tail">
              {safeEntity.name}
            </Text>
            <Text style={[styles.entityDetail, { color: theme.text + "CC" }]}>
              {safeEntity.description || "No description"}
            </Text>
          </>
        )
      case "item":
        return (
          <>
            <Text style={[styles.entityName, { color: theme.text }]} numberOfLines={1} ellipsizeMode="tail">
              {safeEntity.name}
            </Text>
            <Text style={[styles.entityDetail, { color: theme.text + "CC" }]}>
              {safeEntity.description || "No description"}
            </Text>
            <Text style={[styles.entityDetail, { color: theme.text + "CC" }]}>
              Status: {safeEntity.status || "Not specified"}
            </Text>
          </>
        )
      default:
        return <Text style={[styles.entityName, { color: theme.text }]}>{safeEntity.name}</Text>
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
