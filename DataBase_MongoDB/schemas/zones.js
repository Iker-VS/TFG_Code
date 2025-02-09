db.createCollection("zones", {
    validator: {
      $jsonSchema: {
        bsonType: "object",
        required: [ "name", "propertyId" ],
        properties: {
          _id: {
            bsonType: "objectId"
          },
          name: {
            bsonType: "string",
            description: "Nombre de la zona"
          },
          propertyId: {
            bsonType: "objectId",
            description: "Referencia a la propiedad asociada"
          },
          userId: {
            bsonType: "objectId",
            description: "Referencia al usuario asociado a la zona (opcional)"
          },
          parentZoneId: {
            bsonType: "objectId",
            description: "Referencia a la zona padre (opcional)"
          }
        }
      }
    }
  });
  