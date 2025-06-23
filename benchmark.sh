#!/bin/bash

set -euo pipefail

HOST="http://127.0.0.1:8080"
DURATION="30s"
THREADS=12
CONNECTIONS=100

echo "Benchmarking Xpress server at $HOST with $CONNECTIONS clients for $DURATION each."

##############################
# GET /
##############################
echo -e "\n===> GET / (static HTML)"
wrk -t$THREADS -c$CONNECTIONS -d$DURATION $HOST/


##############################
# GET /users
##############################
echo -e "\n===> GET /users"
wrk -t$THREADS -c$CONNECTIONS -d$DURATION $HOST/users


##############################
# POST /users with random emails
##############################

cat > post.lua <<'EOF'
math.randomseed(os.time())

wrk.method = "POST"
wrk.headers["Content-Type"] = "application/json"

function generate_email()
  local prefix = "user" .. math.random(1000000, 9999999)
  return prefix .. "@example.com"
end

function request()
  local user = {
    name = "benchmark_user",
    age = 25,
    email = generate_email()
  }

  local body = string.format(
    '{"name":"%s","age":%d,"email":"%s"}',
    user.name, user.age, user.email
  )

  wrk.body = body
  return wrk.format(nil, "/users")
end
EOF

echo -e "\n===> POST /users"
wrk -t$THREADS -c$CONNECTIONS -d$DURATION -s post.lua $HOST

rm post.lua

echo -e "\n✅ Benchmark complete.