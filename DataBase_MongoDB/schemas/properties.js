db.createCollection("properties", {
    validator: {
      $jsonSchema: {
        bsonType: "object",
        required: [ "name", "groupId" ],
        properties: {
          _id: {
            bsonType: "objectId"
          },
          name: {
            bsonType: "string",
            description: "Nombre de la propiedad"
          },
          direction: {
            bsonType: "string", 
            description: "Dirección (opcional)"
          },
          groupId: {
            bsonType: "objectId",
            description: "Referencia al grupo asociado"
          },
          userId: {
            bsonType: "objectId", 
            description: "Referencia al usuario que registró la propiedad (opcional)"
          }
        }
      }
    }
  });
  