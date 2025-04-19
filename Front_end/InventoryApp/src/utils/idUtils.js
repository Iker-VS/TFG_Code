/**
 * Normaliza un ID de MongoDB para asegurar que siempre se trabaje con un string
 * Maneja tanto IDs simples como objetos con formato {$oid: "..."}
 *
 * @param {string|object} id - El ID a normalizar
 * @returns {string|null} - El ID normalizado como string, o null si no es válido
 */
export const normalizeId = (id) => {
  if (!id) return null

  // Si es un objeto con formato MongoDB {$oid: "..."}
  if (typeof id === "object") {
    if (id.$oid) return id.$oid

    // Si es otro tipo de objeto, intentar convertirlo a string
    try {
      return String(id)
    } catch (e) {
      console.error("Error normalizing ID:", e)
      return null
    }
  }

  // Si ya es un string u otro tipo primitivo
  return String(id)
}

/**
 * Verifica si un ID es válido (no es null, undefined o string vacío)
 *
 * @param {any} id - El ID a verificar
 * @returns {boolean} - true si el ID es válido, false en caso contrario
 */
export const isValidId = (id) => {
  const normalizedId = normalizeId(id)
  return normalizedId !== null && normalizedId !== ""
}

/**
 * Función para reintentar operaciones que pueden fallar por condiciones de carrera
 *
 * @param {Function} operation - La operación a ejecutar
 * @param {number} maxRetries - Número máximo de reintentos
 * @param {number} delay - Retraso entre reintentos en ms
 * @returns {Promise<any>} - El resultado de la operación
 */
export const retryOperation = async (operation, maxRetries = 3, delay = 500) => {
  let lastError

  for (let i = 0; i < maxRetries; i++) {
    try {
      return await operation()
    } catch (error) {
      console.log(`Retry ${i + 1}/${maxRetries} failed:`, error)
      lastError = error

      // Esperar antes de reintentar
      await new Promise((resolve) => setTimeout(resolve, delay))
    }
  }

  throw lastError
}
