db.createCollection("items", {
    validator: {
      $jsonSchema: {
        bsonType: "object",
        required: [ "name", "zoneId" ],
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
          values: {
            bsonType: "array",
            description: "Lista de valores asociados al objeto",
            items: {
              bsonType: "object",
              required: [ "name", "value" ],
              properties: {
                name: {
                  bsonType: "string",
                  description: "Nombre del valor"
                },
                value: {
                  bsonType: ["string", "number", "date"],
                  description: "Valor almacenado en formato texto, numbero o fecha"
                }
              }
            }
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
  