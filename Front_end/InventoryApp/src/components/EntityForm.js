"use client"

import { useState, useContext } from "react"
import {
  View,
  Text,
  TextInput,
  StyleSheet,
  TouchableOpacity,
  ScrollView,
  KeyboardAvoidingView,
  Platform,
} from "react-native"
import { ThemeContext } from "../context/ThemeContext"

const EntityForm = ({ type, initialData = {}, onSubmit, onCancel }) => {
  const { theme } = useContext(ThemeContext)
  const [formData, setFormData] = useState(initialData)

  const handleChange = (key, value) => {
    setFormData({ ...formData, [key]: value })
  }

  const renderFields = () => {
    switch (type) {
      case "group":
        return (
          <>
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
                placeholder="Nombre del grupo"
                placeholderTextColor={theme.text + "80"}
                value={formData.name || ""}
                onChangeText={(text) => handleChange("name", text)}
              />
            </View>
            <View style={styles.inputContainer}>
              <Text style={[styles.label, { color: theme.text }]}>Descripción</Text>
              <TextInput
                style={[
                  styles.textArea,
                  {
                    backgroundColor: theme.card,
                    color: theme.text,
                    borderColor: theme.border,
                  },
                ]}
                placeholder="Descripción del grupo"
                placeholderTextColor={theme.text + "80"}
                value={formData.description || ""}
                onChangeText={(text) => handleChange("description", text)}
                multiline
                numberOfLines={4}
                textAlignVertical="top"
              />
            </View>
          </>
        )
      case "property":
        return (
          <>
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
                placeholder="Nombre de la propiedad"
                placeholderTextColor={theme.text + "80"}
                value={formData.name || ""}
                onChangeText={(text) => handleChange("name", text)}
              />
            </View>
            <View style={styles.inputContainer}>
              <Text style={[styles.label, { color: theme.text }]}>Dirección</Text>
              <TextInput
                style={[
                  styles.input,
                  {
                    backgroundColor: theme.card,
                    color: theme.text,
                    borderColor: theme.border,
                  },
                ]}
                placeholder="Dirección de la propiedad"
                placeholderTextColor={theme.text + "80"}
                value={formData.address || ""}
                onChangeText={(text) => handleChange("address", text)}
              />
            </View>
            <View style={styles.inputContainer}>
              <Text style={[styles.label, { color: theme.text }]}>Descripción</Text>
              <TextInput
                style={[
                  styles.textArea,
                  {
                    backgroundColor: theme.card,
                    color: theme.text,
                    borderColor: theme.border,
                  },
                ]}
                placeholder="Descripción de la propiedad"
                placeholderTextColor={theme.text + "80"}
                value={formData.description || ""}
                onChangeText={(text) => handleChange("description", text)}
                multiline
                numberOfLines={4}
                textAlignVertical="top"
              />
            </View>
          </>
        )
      case "zone":
        return (
          <>
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
                placeholder="Nombre de la zona"
                placeholderTextColor={theme.text + "80"}
                value={formData.name || ""}
                onChangeText={(text) => handleChange("name", text)}
              />
            </View>
            <View style={styles.inputContainer}>
              <Text style={[styles.label, { color: theme.text }]}>Descripción</Text>
              <TextInput
                style={[
                  styles.textArea,
                  {
                    backgroundColor: theme.card,
                    color: theme.text,
                    borderColor: theme.border,
                  },
                ]}
                placeholder="Descripción de la zona"
                placeholderTextColor={theme.text + "80"}
                value={formData.description || ""}
                onChangeText={(text) => handleChange("description", text)}
                multiline
                numberOfLines={4}
                textAlignVertical="top"
              />
            </View>
          </>
        )
      case "item":
        return (
          <>
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
                placeholder="Nombre del objeto"
                placeholderTextColor={theme.text + "80"}
                value={formData.name || ""}
                onChangeText={(text) => handleChange("name", text)}
              />
            </View>
            <View style={styles.inputContainer}>
              <Text style={[styles.label, { color: theme.text }]}>Descripción</Text>
              <TextInput
                style={[
                  styles.textArea,
                  {
                    backgroundColor: theme.card,
                    color: theme.text,
                    borderColor: theme.border,
                  },
                ]}
                placeholder="Descripción del objeto"
                placeholderTextColor={theme.text + "80"}
                value={formData.description || ""}
                onChangeText={(text) => handleChange("description", text)}
                multiline
                numberOfLines={4}
                textAlignVertical="top"
              />
            </View>
            <View style={styles.inputContainer}>
              <Text style={[styles.label, { color: theme.text }]}>Estado</Text>
              <TextInput
                style={[
                  styles.input,
                  {
                    backgroundColor: theme.card,
                    color: theme.text,
                    borderColor: theme.border,
                  },
                ]}
                placeholder="Estado del objeto"
                placeholderTextColor={theme.text + "80"}
                value={formData.status || ""}
                onChangeText={(text) => handleChange("status", text)}
              />
            </View>
          </>
        )
      case "user":
        return (
          <>
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
                placeholder="Nombre de usuario"
                placeholderTextColor={theme.text + "80"}
                value={formData.name || ""}
                onChangeText={(text) => handleChange("name", text)}
              />
            </View>
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
                placeholder="Correo electrónico"
                placeholderTextColor={theme.text + "80"}
                value={formData.email || ""}
                onChangeText={(text) => handleChange("email", text)}
                keyboardType="email-address"
                autoCapitalize="none"
              />
            </View>
          </>
        )
      default:
        return null
    }
  }

  return (
    <KeyboardAvoidingView behavior={Platform.OS === "ios" ? "padding" : "height"} style={{ flex: 1 }}>
      <ScrollView contentContainerStyle={[styles.container, { backgroundColor: theme.background }]}>
        <Text style={[styles.title, { color: theme.text }]}>
          {initialData.id ? "Editar" : "Crear"}{" "}
          {type === "group"
            ? "Grupo"
            : type === "property"
              ? "Propiedad"
              : type === "zone"
                ? "Zona"
                : type === "item"
                  ? "Objeto"
                  : "Usuario"}
        </Text>

        {renderFields()}

        <View style={styles.buttonContainer}>
          <TouchableOpacity
            style={[styles.button, styles.cancelButton, { borderColor: theme.border }]}
            onPress={onCancel}
          >
            <Text style={[styles.buttonText, { color: theme.text }]}>Cancelar</Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.button, styles.submitButton, { backgroundColor: theme.primary }]}
            onPress={() => onSubmit(formData)}
          >
            <Text style={[styles.buttonText, { color: "#fff" }]}>{initialData.id ? "Guardar" : "Crear"}</Text>
          </TouchableOpacity>
        </View>
      </ScrollView>
    </KeyboardAvoidingView>
  )
}

const styles = StyleSheet.create({
  container: {
    padding: 20,
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
  textArea: {
    minHeight: 100,
    borderWidth: 1,
    borderRadius: 8,
    paddingHorizontal: 15,
    paddingVertical: 10,
    fontSize: 16,
  },
  buttonContainer: {
    flexDirection: "row",
    justifyContent: "space-between",
    marginTop: 20,
  },
  button: {
    flex: 1,
    height: 50,
    borderRadius: 8,
    justifyContent: "center",
    alignItems: "center",
  },
  cancelButton: {
    marginRight: 10,
    borderWidth: 1,
  },
  submitButton: {
    marginLeft: 10,
  },
  buttonText: {
    fontSize: 16,
    fontWeight: "bold",
  },
})

export default EntityForm
