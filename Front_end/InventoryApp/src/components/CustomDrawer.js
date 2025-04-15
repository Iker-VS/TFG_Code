"use client"

import { useContext } from "react"
import { View, Text, TouchableOpacity, StyleSheet } from "react-native"
import { DrawerContentScrollView, DrawerItemList } from "@react-navigation/drawer"
import { Ionicons } from "@expo/vector-icons"
import { AuthContext } from "../context/AuthContext"
import { ThemeContext } from "../context/ThemeContext"

const CustomDrawer = (props) => {
  const { logout, userData } = useContext(AuthContext)
  const { theme, themeMode, toggleTheme } = useContext(ThemeContext)

  return (
    <View style={{ flex: 1, backgroundColor: theme.background }}>
      <DrawerContentScrollView {...props} contentContainerStyle={{ backgroundColor: theme.background }}>
        <View style={[styles.userSection, { backgroundColor: theme.primary }]}>
          <View style={styles.userInfo}>
            <View style={styles.userAvatar}>
              <Text style={styles.userInitial}>{userData?.name ? userData.name.charAt(0).toUpperCase() : "U"}</Text>
            </View>
            <Text style={styles.userName}>{userData?.name || "Usuario"}</Text>
            <Text style={styles.userEmail}>{userData?.email || "usuario@ejemplo.com"}</Text>
          </View>
        </View>
        <View style={{ flex: 1, backgroundColor: theme.background, paddingTop: 10 }}>
          <DrawerItemList {...props} />
        </View>
      </DrawerContentScrollView>

      <View style={[styles.bottomSection, { borderTopColor: theme.border }]}>
        <TouchableOpacity onPress={toggleTheme} style={styles.bottomButton}>
          <View style={styles.bottomButtonContent}>
            <Ionicons name={themeMode === "dark" ? "sunny-outline" : "moon-outline"} size={22} color={theme.text} />
            <Text style={[styles.bottomButtonText, { color: theme.text }]}>
              {themeMode === "dark" ? "Modo Claro" : "Modo Oscuro"}
            </Text>
          </View>
        </TouchableOpacity>

        <TouchableOpacity onPress={logout} style={styles.bottomButton}>
          <View style={styles.bottomButtonContent}>
            <Ionicons name="exit-outline" size={22} color={theme.text} />
            <Text style={[styles.bottomButtonText, { color: theme.text }]}>Cerrar Sesión</Text>
          </View>
        </TouchableOpacity>
      </View>
    </View>
  )
}

const styles = StyleSheet.create({
  userSection: {
    padding: 20,
  },
  userInfo: {
    alignItems: "center",
  },
  userAvatar: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: "rgba(255, 255, 255, 0.3)",
    justifyContent: "center",
    alignItems: "center",
    marginBottom: 10,
  },
  userInitial: {
    fontSize: 40,
    color: "#fff",
    fontWeight: "bold",
  },
  userName: {
    color: "#fff",
    fontSize: 18,
    fontWeight: "bold",
  },
  userEmail: {
    color: "#fff",
    fontSize: 14,
    marginTop: 5,
  },
  bottomSection: {
    padding: 20,
    borderTopWidth: 1,
  },
  bottomButton: {
    paddingVertical: 15,
  },
  bottomButtonContent: {
    flexDirection: "row",
    alignItems: "center",
  },
  bottomButtonText: {
    fontSize: 15,
    marginLeft: 10,
  },
})

export default CustomDrawer
