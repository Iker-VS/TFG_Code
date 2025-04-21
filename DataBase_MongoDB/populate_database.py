import random
import string
import hashlib
from datetime import datetime, timedelta

from pymongo import MongoClient
from faker import Faker

# Configuración de la conexión
MONGO_URI = "mongodb://localhost:27017"
DB_NAME = "TFG"

# Cantidades a generar (ajusta según tus necesidades)
NUM_USERS = 10
NUM_GROUPS = 3
PROPERTIES_PER_GROUP = (2, 4)  # rango aleatorio (min, max)
ZONES_PER_PROPERTY = (2, 3)
ITEMS_PER_ZONE = (3, 6)
LOGS_PER_USER = 5

# Inicializar Faker y cliente MongoDB
fake = Faker()
client = MongoClient(MONGO_URI)
db = client[DB_NAME]

# Limpiar colecciones existentes
def clear_collections():
    for col in ["users", "groups", "properties", "zones", "items", "logs", "userGroup"]:
        db[col].drop()
    print("Colecciones limpiadas.")

# Funciones auxiliares

def random_hash():
    pwd = fake.password(length=10)
    return hashlib.sha256(pwd.encode()).hexdigest()

# Generación de datos

def create_users(n):
    users = []
    # Crear usuario admin
    admin = {
        "mail": "admin@ejemplo.com",
        "passwordHash": random_hash(),
        "name": "Administrador",
        "admin": True
    }
    res = db.users.insert_one(admin)
    users.append({**admin, "_id": res.inserted_id})

    for _ in range(n - 1):
        user = {
            "mail": fake.unique.email(),
            "passwordHash": random_hash(),
            "name": fake.name(),
        }
        res = db.users.insert_one(user)
        users.append({**user, "_id": res.inserted_id})
    print(f"{len(users)} usuarios creados.")
    return users


def create_groups(n):
    groups = []
    for _ in range(n):
        code = ''.join(random.choices(string.ascii_letters + string.digits, k=8))
        tags = fake.words(nb=random.randint(1, 5), unique=True)
        group = {
            "name": fake.company(),
            "userCount": 0,
            "groupCode": code,
            "tags": tags
        }
        res = db.groups.insert_one(group)
        groups.append({**group, "_id": res.inserted_id})
    print(f"{len(groups)} grupos creados.")
    return groups


def assign_users_to_groups(users, groups):
    assignments = []
    for group in groups:
        members = random.sample(users, k=random.randint(2, len(users)))
        for user in members:
            ug = {"groupId": group["_id"], "userId": user["_id"]}
            db.userGroup.insert_one(ug)
            assignments.append(ug)
        db.groups.update_one({"_id": group["_id"]}, {"$set": {"userCount": len(members)}})
    print(f"{len(assignments)} asignaciones usuario-grupo creadas.")
    return assignments


def create_properties(groups):
    properties = []
    for group in groups:
        count = random.randint(*PROPERTIES_PER_GROUP)
        for _ in range(count):
            prop = {
                "name": fake.street_name(),
                **({"direction": fake.address()} if random.random() < 0.5 else {}),
                "groupId": group["_id"],
                **({"userId": random.choice(groups)["_id"]} if random.random() < 0.5 else {})
            }
            res = db.properties.insert_one(prop)
            properties.append({**prop, "_id": res.inserted_id})
    print(f"{len(properties)} propiedades creadas.")
    return properties


def create_zones(properties, users):
    zones = []
    for prop in properties:
        count = random.randint(*ZONES_PER_PROPERTY)
        parent = None
        for i in range(count):
            zone = {
                "name": f"Zona {fake.word().capitalize()}",
                "propertyId": prop["_id"],
                **({"userId": random.choice(users)["_id"]} if random.random() < 0.5 else {}),
                **({"parentZoneId": parent["_id"]} if parent and random.random() < 0.3 else {})
            }
            res = db.zones.insert_one(zone)
            zone_record = {**zone, "_id": res.inserted_id}
            zones.append(zone_record)
            if random.random() < 0.3:
                parent = zone_record
    print(f"{len(zones)} zonas creadas.")
    return zones


def create_items(zones, groups):
    items = []
    for zone in zones:
        prop = db.properties.find_one({"_id": zone["propertyId"]})
        group = db.groups.find_one({"_id": prop["groupId"]})
        for _ in range(random.randint(*ITEMS_PER_ZONE)):
            tag_choices = group.get("tags", [])
            item = {
                "name": fake.word().capitalize(),
                **({"description": fake.sentence(nb_words=6)} if random.random() < 0.7 else {}),
                "zoneId": zone["_id"],
                **({"tags": random.sample(tag_choices, k=random.randint(1, len(tag_choices))) } if tag_choices else {})
            }
            res = db.items.insert_one(item)
            items.append({**item, "_id": res.inserted_id})
    print(f"{len(items)} items creados.")
    return items


def create_logs(groups, users):
    logs = []
    for _ in range(LOGS_PER_USER * len(users)):
        user = random.choice(users)
        group = random.choice(groups)
        time = datetime.now() - timedelta(days=random.randint(0, 30), hours=random.randint(0,23))
        log = {
            "description": fake.sentence(nb_words=8),
            "time": time,
            "groupId": group["_id"],
            "userId": user["_id"]
        }
        res = db.logs.insert_one(log)
        logs.append({**log, "_id": res.inserted_id})
    print(f"{len(logs)} logs creados.")
    return logs


def main():
    clear_collections()
    users = create_users(NUM_USERS)
    groups = create_groups(NUM_GROUPS)
    assign_users_to_groups(users, groups)
    properties = create_properties(groups)
    zones = create_zones(properties, users)
    create_items(zones, groups)
    create_logs(groups, users)
    print("Población de la base de datos completada.")

if __name__ == "__main__":
    main()
