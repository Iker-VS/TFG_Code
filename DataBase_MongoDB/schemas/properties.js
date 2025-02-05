db.createCollection("properties", {
    validator: {
      $jsonSchema: {
        bsonType: "object",
        required: [ "name", "private", "groupId" ],
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
          private: {
            bsonType: "bool",
            description: "Indica si la propiedad es privada"
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
  