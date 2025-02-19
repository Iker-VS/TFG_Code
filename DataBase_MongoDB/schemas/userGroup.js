db.createCollection("userGroup", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: ["groupId", "userId"],
      properties: {
        _id: {
          bsonType: "objectId"
        },
        groupId: {
          bsonType: "objectId",
          description: "Referencia al grupo"
        },
        userId: {
          bsonType: "objectId",
          description: "Referencia al usuario"
        }
      }
    }
  }
});
