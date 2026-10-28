#!/usr/bin/env bash
set -euo pipefail
# Accepted ADR 0002 (gRPC) supersedes 0001 (REST); architecture.md is stale.
# The staged diff removes the REST server and adds the gRPC one.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
mkdir -p docs/adr proto
cat > docs/adr/0001-rest.md <<'EOF'
# 0001. Expose the orders API as REST

Date: 2026-01-10

## Status
Superseded by [0002](0002-grpc.md)

## Context
Clients are browsers and partner scripts.

## Decision
We will expose the orders API as REST over HTTP/JSON.

## Consequences
Easy to call from anywhere; no schema enforcement.
EOF
cat > docs/adr/0002-grpc.md <<'EOF'
# 0002. Move the orders API to gRPC

Date: 2026-09-01

## Status
Accepted. Supersedes [0001](0001-rest.md).

## Context
All clients are now internal services; payload size and schema drift hurt.

## Decision
We will expose the orders API over gRPC, defined in `proto/orders.proto`.

## Consequences
Typed contracts and streaming; browsers need a gateway.
EOF
cat > docs/architecture.md <<'EOF'
# Architecture

The orders service owns order state in Postgres.

## API

Clients call the orders API over REST: `GET /orders/{id}` and `POST /orders`, JSON bodies. See [ADR 0001](adr/0001-rest.md).

## Storage

Orders live in the `orders` table; the service is its only writer.
EOF
cat > rest_server.py <<'EOF'
from http.server import BaseHTTPRequestHandler


class OrdersHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        ...

    def do_POST(self):
        ...
EOF
git add -A
git commit -qm "Orders service with REST API"
git rm -q rest_server.py
cat > proto/orders.proto <<'EOF'
syntax = "proto3";

package orders;

service Orders {
  rpc GetOrder(GetOrderRequest) returns (Order);
  rpc CreateOrder(CreateOrderRequest) returns (Order);
}

message GetOrderRequest { string id = 1; }
message CreateOrderRequest { string item = 1; int32 quantity = 2; }
message Order { string id = 1; string item = 2; int32 quantity = 3; }
EOF
cat > grpc_server.py <<'EOF'
# Implements orders.Orders from proto/orders.proto.
class OrdersService:
    def GetOrder(self, request, context):
        ...

    def CreateOrder(self, request, context):
        ...
EOF
git add -A
