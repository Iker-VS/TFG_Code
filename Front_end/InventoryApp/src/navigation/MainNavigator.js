"use client"

import { useContext } from "react"
import { createStackNavigator } from "@react-navigation/stack"
import { AuthContext } from "../context/AuthContext"
import AuthScreen from "../screens/AuthScreen"
import AppNavigator from "./AppNavigator"
import LoadingScreen from "../screens/LoadingScreen"

const Stack = createStackNavigator()

const MainNavigator = () => {
  const { isLoading, userToken } = useContext(AuthContext)

  if (isLoading) {
    return <LoadingScreen />
  }

  return (
    <Stack.Navigator screenOptions={{ headerShown: false }}>
      {userToken ? (
        <Stack.Screen name="App" component={AppNavigator} />
      ) : (
        <Stack.Screen name="Auth" component={AuthScreen} />
      )}
    </Stack.Navigator>
  )
}

export default MainNavigator
