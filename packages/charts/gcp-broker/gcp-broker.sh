#!/bin/sh
export PORT="8080"
export ROOT_SERVICE_ACCOUNT_JSON=$(cat /Users/jiayihu/Desktop/Repo/fedra-thesis/key-gcp-broker.json)
export SECURITY_USER_NAME="root"
export SECURITY_USER_PASSWORD="root"
export DB_HOST="35.205.8.126"
export DB_USERNAME="broker"
export DB_PASSWORD="jzaL5AMH6fG6epfi"
export DB_PORT="3306"
export DB_NAME="servicebroker"
export GSB_BROKERPAK_CONFIG="{}"
export GSB_SERVICE_CONFIG="{}"

./gcp-service-broker-bin serve
