import { NavigationContainer } from "@react-navigation/native"
import { AuthProvider } from "./src/context/AuthContext"
import { ThemeProvider } from "./src/context/ThemeContext"
import MainNavigator from "./src/navigation/MainNavigator"
import { StatusBar } from "expo-status-bar"

export default function App() {
  return (
    <AuthProvider>
      <ThemeProvider>
        <NavigationContainer>
          <StatusBar style="auto" />
          <MainNavigator />
        </NavigationContainer>
      </ThemeProvider>
    </AuthProvider>
  )
}
