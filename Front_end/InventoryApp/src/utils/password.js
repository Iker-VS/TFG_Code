import { SHA256 } from "crypto-js"

/**
 * Hashes a plain text password using SHA-256
 * This function creates a consistent hash that can be verified by the server
 *
 * @param {string} plainPassword - The plain text password to hash
 * @returns {Promise<string>} - The hashed password
 */
export const hashPassword = async (plainPassword) => {
  try {
    // For login, we need a consistent hash without a random salt
    // This is because the server will compare the hash directly
    const hashedPassword = SHA256(plainPassword).toString()

    console.log("Password hashed successfully with SHA-256")
    return hashedPassword
  } catch (error) {
    console.error("Error hashing password:", error)
    throw new Error("Failed to hash password")
  }
}

/**
 * Validates if a password meets security requirements
 *
 * @param {string} password - The password to validate
 * @returns {boolean} - Whether the password is valid
 */
export const validatePassword = (password) => {
  // Password must be at least 8 characters long
  if (!password || password.length < 8) {
    return false
  }

  // Password should contain at least one uppercase letter, one lowercase letter, and one number
  const hasUppercase = /[A-Z]/.test(password)
  const hasLowercase = /[a-z]/.test(password)
  const hasNumber = /[0-9]/.test(password)

  return hasUppercase && hasLowercase && hasNumber
}
