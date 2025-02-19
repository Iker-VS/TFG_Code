db.createCollection("groups", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: ["name", "userCount", "groupCode"],
      properties: {
        _id: {
          bsonType: "objectId"
        },
        name: {
          bsonType: "string",
          description: "Nombre del grupo"
        },
        userCount: {
          bsonType: "int",
          description: "Cantidad actual de usuarios en el grupo"
        },
        userMax: {
          bsonType: "int",
          description: "Cantidad máxima de usuarios permitidos (puede ser nulo)"
        },
        groupCode: {
          bsonType: "string",
          description: "Código único del grupo (8 caracteres)",
          pattern: "^[A-Za-z0-9]{8}$"
        },
        tags: {
          bsonType: "array",
          description: "Lista de tags definidos para el grupo",
          items: {
            bsonType: "string",
          }
        }
      }
    }
  }
});