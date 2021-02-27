#!/bin/sh
export PORT="8080"
export ROOT_SERVICE_ACCOUNT_JSON=$(cat /Users/jiayihu/Desktop/Repo/fedra-thesis/key-gcp-broker.json)
export SECURITY_USER_NAME="root"
export SECURITY_USER_PASSWORD="root"
source ./.env
# export DB_HOST="db-ip-not-here"
# export DB_USERNAME="username-not-here"
# export DB_PASSWORD="password-not-here"
export DB_PORT="3306"
export DB_NAME="servicebroker"
export GSB_BROKERPAK_CONFIG="{}"
export GSB_SERVICE_CONFIG="{}"

./gcp-service-broker-bin serve
