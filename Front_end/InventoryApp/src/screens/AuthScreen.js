"use client"

import { useState, useContext } from "react"
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  KeyboardAvoidingView,
  Platform,
  ScrollView,
  Alert,
} from "react-native"
import { AuthContext } from "../context/AuthContext"
import { ThemeContext } from "../context/ThemeContext"

const AuthScreen = () => {
  const [isLogin, setIsLogin] = useState(true)
  const [name, setName] = useState("")
  const [email, setEmail] = useState("")
  const [password, setPassword] = useState("")
  const [confirmPassword, setConfirmPassword] = useState("")

  const { login, register, error } = useContext(AuthContext)
  const { theme } = useContext(ThemeContext)

  const validateForm = () => {
    if (isLogin) {
      if (!email.trim() || !password.trim()) {
        Alert.alert("Error", "Por favor completa todos los campos")
        return false
      }
    } else {
      if (!name.trim() || !email.trim() || !password.trim() || !confirmPassword.trim()) {
        Alert.alert("Error", "Por favor completa todos los campos")
        return false
      }

      if (password !== confirmPassword) {
        Alert.alert("Error", "Las contraseñas no coinciden")
        return false
      }
    }
    return true
  }

  const handleSubmit = () => {
    if (!validateForm()) return

    if (isLogin) {
      login(email, password)
    } else {
      register(name, email, password)
    }
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      style={[styles.container, { backgroundColor: theme.background }]}
    >
      <ScrollView contentContainerStyle={styles.scrollContainer}>
        <View style={styles.formContainer}>
          <Text style={[styles.title, { color: theme.text }]}>{isLogin ? "Iniciar Sesión" : "Crear Cuenta"}</Text>

          {error && <Text style={styles.errorText}>{error}</Text>}

          {!isLogin && (
            <View style={styles.inputContainer}>
              <Text style={[styles.label, { color: theme.text }]}>Nombre</Text>
              <TextInput
                style={[
                  styles.input,
                  {
                    backgroundColor: theme.card,
                    color: theme.text,
                    borderColor: theme.border,
                  },
                ]}
                placeholder="Ingresa tu nombre"
                placeholderTextColor={theme.text + "80"}
                value={name}
                onChangeText={setName}
              />
            </View>
          )}

          <View style={styles.inputContainer}>
            <Text style={[styles.label, { color: theme.text }]}>Correo Electrónico</Text>
            <TextInput
              style={[
                styles.input,
                {
                  backgroundColor: theme.card,
                  color: theme.text,
                  borderColor: theme.border,
                },
              ]}
              placeholder="Ingresa tu correo"
              placeholderTextColor={theme.text + "80"}
              value={email}
              onChangeText={setEmail}
              keyboardType="email-address"
              autoCapitalize="none"
            />
          </View>

          <View style={styles.inputContainer}>
            <Text style={[styles.label, { color: theme.text }]}>Contraseña</Text>
            <TextInput
              style={[
                styles.input,
                {
                  backgroundColor: theme.card,
                  color: theme.text,
                  borderColor: theme.border,
                },
              ]}
              placeholder="Ingresa tu contraseña"
              placeholderTextColor={theme.text + "80"}
              value={password}
              onChangeText={setPassword}
              secureTextEntry
            />
          </View>

          {!isLogin && (
            <View style={styles.inputContainer}>
              <Text style={[styles.label, { color: theme.text }]}>Confirmar Contraseña</Text>
              <TextInput
                style={[
                  styles.input,
                  {
                    backgroundColor: theme.card,
                    color: theme.text,
                    borderColor: theme.border,
                  },
                ]}
                placeholder="Confirma tu contraseña"
                placeholderTextColor={theme.text + "80"}
                value={confirmPassword}
                onChangeText={setConfirmPassword}
                secureTextEntry
              />
            </View>
          )}

          <TouchableOpacity style={[styles.button, { backgroundColor: theme.primary }]} onPress={handleSubmit}>
            <Text style={styles.buttonText}>{isLogin ? "Iniciar Sesión" : "Crear Cuenta"}</Text>
          </TouchableOpacity>

          <TouchableOpacity style={styles.switchButton} onPress={() => setIsLogin(!isLogin)}>
            <Text style={[styles.switchText, { color: theme.primary }]}>
              {isLogin ? "¿No tienes cuenta? Crear una" : "¿Ya tienes cuenta? Iniciar sesión"}
            </Text>
          </TouchableOpacity>
        </View>
      </ScrollView>
    </KeyboardAvoidingView>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  scrollContainer: {
    flexGrow: 1,
    justifyContent: "center",
    padding: 20,
  },
  formContainer: {
    width: "100%",
    maxWidth: 400,
    alignSelf: "center",
  },
  title: {
    fontSize: 24,
    fontWeight: "bold",
    marginBottom: 20,
    textAlign: "center",
  },
  inputContainer: {
    marginBottom: 15,
  },
  label: {
    marginBottom: 5,
    fontSize: 16,
  },
  input: {
    height: 50,
    borderWidth: 1,
    borderRadius: 8,
    paddingHorizontal: 15,
    fontSize: 16,
  },
  button: {
    height: 50,
    borderRadius: 8,
    justifyContent: "center",
    alignItems: "center",
    marginTop: 20,
  },
  buttonText: {
    color: "#fff",
    fontSize: 16,
    fontWeight: "bold",
  },
  switchButton: {
    marginTop: 20,
    alignItems: "center",
  },
  switchText: {
    fontSize: 16,
  },
  errorText: {
    color: "#FF3B30",
    marginBottom: 15,
    textAlign: "center",
  },
})

export default AuthScreen
