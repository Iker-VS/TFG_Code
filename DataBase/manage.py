import hashlib
import getpass
import sys
from pymongo import MongoClient

# Configuración de la conexión
MONGO_URI = "mongodb://localhost:27017"  # Cambia esto si usas credenciales o URL distinta
DB_NAME = "TFG"  # Reemplaza con el nombre de tu base de datos

# Conexión a MongoDB
client = MongoClient(MONGO_URI)
db = client[DB_NAME]
usuarios = db.get_collection("users")

# Función de hashing: SHA256 de la cadena dada
def hash_password(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()

# Validación del esquema (simple, basada en tu schema)
def validate_user(user):
    if not isinstance(user.get("name"), str) or len(user["name"]) < 1:
        return False, "Nombre inválido."
    if not isinstance(user.get("mail"), str) or "@" not in user["mail"]:
        return False, "Email inválido."
    if not isinstance(user.get("passwordHash"), str) or len(user["passwordHash"]) != 64:
        return False, "La contraseña no está correctamente hasheada (SHA-256 produce 64 caracteres hex)."
    return True, "OK"

# Recoger datos desde terminal
try:
    print("=== Crear usuario administrador ===")
    name = input("Nombre: ").strip()
    mail = input("Email: ").strip()
    password = getpass.getpass("Contraseña: ")
    confirm = getpass.getpass("Confirmar contraseña: ")

    if password != confirm:
        print("❌ Las contraseñas no coinciden.")
        sys.exit(1)

    # Verificar que no exista ya el email
    if usuarios.find_one({"mail": mail}):
        print("❌ Ya existe un usuario con ese email.")
        sys.exit(1)

    # Crear usuario con SHA-256
    hashed = hash_password(password)
    user = {
        "mail": mail,
        "passwordHash": hashed,
        "name": name,
        "admin": True  # Establecer el rol como administrador
    }

    valid, message = validate_user(user)
    if not valid:
        print("❌ Error de validación:", message)
        sys.exit(1)

    # Insertar en la base de datos
    usuarios.insert_one(user)
    print("✅ Usuario administrador creado con éxito. (Password hashed con SHA-256)")

except Exception as e:
    print("❌ Error:", str(e))
    sys.exit(1)
