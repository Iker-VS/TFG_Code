db.createCollection("logs", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: ["description", "time", "groupId", "userId"],
      properties: {
        _id: {
          bsonType: "objectId"
        },
        description: {
          bsonType: "string",
          description: "Descripción del evento de log"
        },
        time: {
          bsonType: "date",
          description: "Fecha y hora del evento"
        },
        groupId: {
          bsonType: "objectId",
          description: "Referencia al grupo asociado"
        },
        userId: {
          bsonType: "objectId",
          description: "Referencia al usuario que realizó la acción"
        }
      }
    }
  }
});
