"use client"

import { useState, useContext, useEffect } from "react"
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
import { validatePassword } from "../utils/password"
import { Ionicons } from "@expo/vector-icons"

const AuthScreen = () => {
  const [isLogin, setIsLogin] = useState(true)
  const [name, setName] = useState("")
  const [email, setEmail] = useState("")
  const [password, setPassword] = useState("")
  const [confirmPassword, setConfirmPassword] = useState("")
  const [passwordsMatch, setPasswordsMatch] = useState(null)
  const [passwordStrength, setPasswordStrength] = useState(null)

  const { login, register, error, clearErrors, registrationSuccess, setRegistrationSuccess } = useContext(AuthContext)
  const { theme } = useContext(ThemeContext)

  // Autocompletar campos después de un registro exitoso
  useEffect(() => {
    if (registrationSuccess && isLogin) {
      // Auto-fill the login form with registration credentials
      setEmail(registrationSuccess.email || "")
      setPassword(registrationSuccess.password || "")

      // Optional: Show a toast or message to inform the user
      console.log("Login form auto-filled with registration credentials")
    }
  }, [registrationSuccess, isLogin])

  // Clear password match status when switching between login and register
  useEffect(() => {
    setPasswordsMatch(null)
    setPasswordStrength(null)
    clearErrors()
  }, [isLogin])

  // Check password match and strength when either password changes
  useEffect(() => {
    if (!isLogin) {
      // Only check password match if both fields have values
      if (password && confirmPassword) {
        setPasswordsMatch(password === confirmPassword)
      } else {
        setPasswordsMatch(null)
      }

      // Check password strength if password field has a value
      if (password) {
        setPasswordStrength(validatePassword(password))
      } else {
        setPasswordStrength(null)
      }
    }
  }, [password, confirmPassword, isLogin])

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

      if (!passwordStrength) {
        Alert.alert("Error", "La contraseña debe tener al menos 8 caracteres, una mayúscula, una minúscula y un número")
        return false
      }

      if (!passwordsMatch) {
        Alert.alert("Error", "Las contraseñas no coinciden")
        return false
      }
    }
    return true
  }

  const handleSubmit = async () => {
    if (!validateForm()) return

    if (isLogin) {
      login(email, password)
    } else {
      const success = await register(name, email, password, confirmPassword)
      if (success) {
        // Si el registro fue exitoso, cambiar a la pantalla de login
        setIsLogin(true)
      }
    }
  }

  // Toggle between login and register views
  const toggleAuthMode = () => {
    // Clear form fields
    if (isLogin) {
      setName("")
      // Don't clear email and password if coming from successful registration
      if (!registrationSuccess) {
        setEmail("")
        setPassword("")
      }
      setConfirmPassword("")
    } else {
      setConfirmPassword("")
      // Don't clear registration success data when switching to login
    }

    // Reset password validation states
    setPasswordsMatch(null)
    setPasswordStrength(null)

    // Clear any error messages
    clearErrors()

    // Toggle the auth mode
    setIsLogin(!isLogin)
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      style={[styles.container, { backgroundColor: theme.background }]}
    >
      <ScrollView contentContainerStyle={styles.scrollContainer}>
        <View style={styles.formContainer}>
          <Text style={[styles.title, { color: theme.text }]}>{isLogin ? "Iniciar Sesión" : "Crear Cuenta"}</Text>

          {error && <Text style={[styles.errorText, { color: theme.error }]}>{error}</Text>}

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
            {!isLogin && passwordStrength !== null && (
              <View style={styles.validationContainer}>
                <Ionicons
                  name={passwordStrength ? "checkmark-circle" : "close-circle"}
                  size={16}
                  color={passwordStrength ? theme.success : theme.error}
                />
                <Text style={[styles.validationText, { color: passwordStrength ? theme.success : theme.error }]}>
                  {passwordStrength
                    ? "Contraseña segura"
                    : "La contraseña debe tener al menos 8 caracteres, una mayúscula, una minúscula y un número"}
                </Text>
              </View>
            )}
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
              {passwordsMatch !== null && (
                <View style={styles.validationContainer}>
                  <Ionicons
                    name={passwordsMatch ? "checkmark-circle" : "close-circle"}
                    size={16}
                    color={passwordsMatch ? theme.success : theme.error}
                  />
                  <Text style={[styles.validationText, { color: passwordsMatch ? theme.success : theme.error }]}>
                    {passwordsMatch ? "Las contraseñas coinciden" : "Las contraseñas no coinciden"}
                  </Text>
                </View>
              )}
            </View>
          )}

          <TouchableOpacity style={[styles.button, { backgroundColor: theme.primary }]} onPress={handleSubmit}>
            <Text style={styles.buttonText}>{isLogin ? "Iniciar Sesión" : "Crear Cuenta"}</Text>
          </TouchableOpacity>

          <TouchableOpacity style={styles.switchButton} onPress={toggleAuthMode}>
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
    marginBottom: 15,
    textAlign: "center",
    fontSize: 14,
  },
  validationContainer: {
    flexDirection: "row",
    alignItems: "center",
    marginTop: 5,
  },
  validationText: {
    fontSize: 12,
    marginLeft: 5,
  },
})

export default AuthScreen
