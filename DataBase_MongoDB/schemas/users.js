db.createCollection("users", {
    validator: {
      $jsonSchema: {
        bsonType: "object",
        required: [ "mail", "passwordHash", "name" ],
        properties: {
          _id: {
            bsonType: "objectId"
          },
          mail: {
            bsonType: "string",
            description: "Correo electrónico del usuario",
            pattern: "^.+@.+$"
          },
          passwordHash: {
            bsonType: "string",
            description: "Hash de la contraseña"
          },
          name: {
            bsonType: "string",
            description: "Nombre del usuario"
          },
          admin: {
            bsonType: "bool",
            description: "Indica si el usuario es administrador (opcional)"
          }
        }
      }
    }
  });
  