#!/bin/bash

# Number of random users to generate
USER_COUNT=100

# Function to generate a random string of a given length
generate_random_string() {
    local length=$1
    tr -dc A-Za-z0-9 </dev/urandom | head -c ${length}
}

# Generate random users
for ((i=1; i<=USER_COUNT; i++)); do
    USERNAME=$(generate_random_string 8)
    PASSWORD=$(generate_random_string 12)
    
    curl -X POST "http://localhost:8080/add_user" \
    -H "Content-Type: application/json" \
    -d "{\"username\": \"$USERNAME\", \"password\": \"$PASSWORD\"}"
    
    echo -e "\nUser $i: Username = $USERNAME, Password = $PASSWORD"
done
