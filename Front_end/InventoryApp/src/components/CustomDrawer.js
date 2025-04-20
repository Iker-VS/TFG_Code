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

  // Determinar si el usuario es administrador
  const isAdmin = userData?.admin === true || userData?.role === "admin"

  // Obtener el nombre y correo del usuario
  const userName = userData?.name || "Usuario"
  const userEmail = userData?.email || userData?.mail || "usuario@ejemplo.com"

  return (
    <View style={{ flex: 1, backgroundColor: theme.background }}>
      <DrawerContentScrollView {...props} contentContainerStyle={{ backgroundColor: theme.background }}>
        <View style={[styles.userSection, { backgroundColor: theme.primary }]}>
          <View style={styles.userInfo}>
            <View style={styles.userAvatar}>
              <Text style={styles.userInitial}>{userName ? userName.charAt(0).toUpperCase() : "U"}</Text>
            </View>
            <Text style={styles.userName}>{userName}</Text>
            <Text style={styles.userEmail}>{userEmail}</Text>
          </View>
        </View>
        <View style={{ flex: 1, backgroundColor: theme.background, paddingTop: 10 }}>
          {/* Modificar el DrawerItemList para asegurar que el texto se ajuste correctamente */}
          <DrawerItemList
            {...props}
            labelStyle={{
              fontSize: 15,
              fontWeight: "500",
              marginLeft: -20,
              width: "100%",
            }}
            itemStyle={{
              marginVertical: 5,
              marginHorizontal: 5,
            }}
          />
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
    textAlign: "center",
  },
  userEmail: {
    color: "#fff",
    fontSize: 14,
    marginTop: 5,
    textAlign: "center",
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
