db.createCollection("items", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: ["name", "zoneId"],
      properties: {
        _id: {
          bsonType: "objectId"
        },
        name: {
          bsonType: "string",
          description: "Nombre del objeto"
        },
        description: {
          bsonType: "string",
          description: "Descripción opcional del objeto"
        },
        pictureUrl: {
          bsonType: "string",
          description: "URL de la imagen del objeto (almacenada en la nube)"
        },
        zoneId: {
          bsonType: "objectId",
          description: "Referencia a la zona en la que se encuentra el objeto"
        },
        tags: {
          bsonType: "array",
          description: "Lista de tags asociados al objeto (según los definidos en su grupo)",
          items: {
            bsonType: "string"
          }
        }
      }
    }
  }
});
