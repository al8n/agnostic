#!/bin/bash

# Install jq
sudo apt-get install jq -y

# Query coverage for crate
sleep 60
curl --request GET \
  --url "https://api.codecov.io/api/v2/github/al8n/repos/agnostic/report/tree?branch=0.6.0&depth=1&path=$1" \
  --header "accept: application/json" \
  --header "authorization: Bearer $2" \
  | jq '.[0].coverage | {coverage: (. | ceil)}' > "$1-coverage.json"