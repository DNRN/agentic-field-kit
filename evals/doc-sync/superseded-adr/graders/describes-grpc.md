---
type: llm
weight: 3
focus: { source: file, path: docs/architecture.md }
---

This is the project's architecture doc after a docs sync. The orders API moved from REST to gRPC (ADR 0002 supersedes ADR 0001).

PASS if the API section describes gRPC (the `Orders` service, `proto/orders.proto`) as the current API, links ADR 0002 (`adr/0002-grpc.md`), and does not present REST endpoints as current (mentioning REST only as superseded history is fine).
FAIL if it still presents REST as the current API style, or if it does not link ADR 0002.
