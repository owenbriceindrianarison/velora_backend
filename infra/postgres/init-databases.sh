#!/bin/bash
set -e

# Ce script est exécuté automatiquement par l'image postgres
# au PREMIER démarrage du conteneur (dossier /docker-entrypoint-initdb.d).

databases=(
  velora_auth
  velora_users
  velora_catalog
  velora_media
  velora_playback
  velora_billing
)

for db in "${databases[@]}"; do
  echo ">>> Création de la base : $db"
  psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
    CREATE DATABASE $db;
EOSQL
done