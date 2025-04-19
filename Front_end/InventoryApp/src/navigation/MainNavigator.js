"use client"

import { useContext, useEffect } from "react"
import { createStackNavigator } from "@react-navigation/stack"
import { AuthContext } from "../context/AuthContext"
import { setGlobalAuthContext } from "../services/api"
import AuthScreen from "../screens/AuthScreen"
import AppNavigator from "./AppNavigator"
import LoadingScreen from "../screens/LoadingScreen"

const Stack = createStackNavigator()

// Ensure the app always starts on the login screen
const MainNavigator = () => {
  const authContext = useContext(AuthContext)
  const { isLoading, userToken, authInvalidated, resetAuthInvalidated } = authContext

  // Set the global auth context for API calls
  useEffect(() => {
    setGlobalAuthContext(authContext)
  }, [authContext])

  // Handle auth invalidation
  useEffect(() => {
    if (authInvalidated) {
      console.log("Auth invalidated, resetting flag")
      resetAuthInvalidated()
    }
  }, [authInvalidated, resetAuthInvalidated])

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
