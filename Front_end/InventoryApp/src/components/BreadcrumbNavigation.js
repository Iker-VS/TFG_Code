"use client"

import { useContext } from "react"
import { View, Text, StyleSheet, ScrollView, TouchableOpacity } from "react-native"
import { Ionicons } from "@expo/vector-icons"
import { ThemeContext } from "../context/ThemeContext"

const BreadcrumbNavigation = ({ path, onNavigate }) => {
  const { theme } = useContext(ThemeContext)

  if (!path || path.length === 0) {
    return null
  }

  return (
    <View style={[styles.container, { borderBottomColor: theme.border }]}>
      <ScrollView horizontal showsHorizontalScrollIndicator={false} contentContainerStyle={styles.scrollContent}>
        {path.map((item, index) => (
          <View key={index} style={styles.breadcrumbItem}>
            {index > 0 && (
              <Ionicons name="chevron-forward" size={16} color={theme.text + "80"} style={styles.separator} />
            )}
            <TouchableOpacity onPress={() => onNavigate(index)} style={styles.breadcrumbButton}>
              <Text
                style={[
                  styles.breadcrumbText,
                  {
                    color: index === path.length - 1 ? theme.primary : theme.text + "CC",
                    fontWeight: index === path.length - 1 ? "bold" : "normal",
                  },
                ]}
                numberOfLines={1}
              >
                {item.name}
              </Text>
            </TouchableOpacity>
          </View>
        ))}
      </ScrollView>
    </View>
  )
}

const styles = StyleSheet.create({
  container: {
    borderBottomWidth: 1,
    paddingVertical: 10,
  },
  scrollContent: {
    paddingHorizontal: 15,
  },
  breadcrumbItem: {
    flexDirection: "row",
    alignItems: "center",
  },
  separator: {
    marginHorizontal: 5,
  },
  breadcrumbButton: {
    paddingVertical: 5,
  },
  breadcrumbText: {
    fontSize: 14,
  },
})

export default BreadcrumbNavigation
