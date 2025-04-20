"use client"

import { useContext } from "react"
import { createDrawerNavigator } from "@react-navigation/drawer"
import { AuthContext } from "../context/AuthContext"
import { ThemeContext } from "../context/ThemeContext"
import CustomDrawer from "../components/CustomDrawer"
import MainScreen from "../screens/MainScreen"
import GroupsScreen from "../screens/GroupsScreen"
import GroupDetailScreen from "../screens/GroupDetailScreen"
import UserScreen from "../screens/UserScreen"
import LogsScreen from "../screens/LogsScreen"
import UsersScreen from "../screens/UsersScreen"

const Drawer = createDrawerNavigator()

const AppNavigator = () => {
  const { userData } = useContext(AuthContext)
  const { theme } = useContext(ThemeContext)

  // Verificar si el usuario es administrador usando tanto admin como role
  const isAdmin = userData?.admin === true || userData?.role === "admin"

  return (
    <Drawer.Navigator
      drawerContent={(props) => <CustomDrawer {...props} />}
      screenOptions={{
        headerShown: true,
        headerStyle: {
          backgroundColor: theme.primary,
        },
        headerTintColor: "#fff",
        drawerActiveBackgroundColor: theme.primary,
        drawerActiveTintColor: "#fff",
        drawerInactiveTintColor: theme.text,
        drawerLabelStyle: {
          marginLeft: -25,
          fontSize: 15,
        },
      }}
    >
      <Drawer.Screen name="Home" component={MainScreen} options={{ title: "Inicio" }} />
      <Drawer.Screen name="Groups" component={GroupsScreen} options={{ title: "Mis Grupos" }} />
      <Drawer.Screen
        name="GroupDetail"
        component={GroupDetailScreen}
        options={{
          title: "Detalle de Grupo",
          drawerItemStyle: { display: "none" }, // Ocultar en el drawer
        }}
      />
      <Drawer.Screen name="User" component={UserScreen} options={{ title: "Mi Usuario" }} />
      {isAdmin && (
        <>
          <Drawer.Screen name="Logs" component={LogsScreen} options={{ title: "Logs" }} />
          <Drawer.Screen name="Users" component={UsersScreen} options={{ title: "Usuarios" }} />
        </>
      )}
    </Drawer.Navigator>
  )
}

export default AppNavigator
