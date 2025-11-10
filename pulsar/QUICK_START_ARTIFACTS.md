# Pulsar Artifacts Quick Start Guide

## Quick Links

- **Artifact Registry SQL**: `/opt/singulio-dev/scripts/register-pulsar-artifacts.sql`
- **Schema SQL**: `/opt/singulio-dev/migrations/create_artifacts_schema.sql`
- **gRPC API Spec**: `/opt/singulio-dev/shared/api-specs/openapi/pulsar-grpc.yaml`
- **WebSocket API Spec**: `/opt/singulio-dev/shared/api-specs/asyncapi/pulsar-websocket.yaml`
- **Documentation**: `/opt/singulio-dev/docs/artifacts/pulsar-daemon.md`

## Registered Artifacts

| ID | Name | Category | Version | Status |
|----|------|----------|---------|--------|
| `pulsar-daemon` | Pulsar Terminal Daemon | Infrastructure | 0.1.0 | Production |
| `pulsar-desktop` | Pulsar Desktop Terminal | Application | 0.1.0 | Production |
| `terminal-wasm` | Terminal WebAssembly Core | Library | 0.1.0 | Production |
| `tft-core` | Terminal Framework Core | Library | 0.1.0 | Production |
| `tft-transports` | Terminal Framework Transports | Library | 0.1.0 | Production |
| `pulsar-protocols` | Pulsar Protocol Specifications | Specification | 1.0.0 | Production |

## Deploy to Database

```bash
# 1. Set database URL
export DATABASE_URL="postgres://singulio:singulio_dev_pass@localhost:5432/singulio"

# 2. Create schema (run once)
psql $DATABASE_URL -f /opt/singulio-dev/migrations/create_artifacts_schema.sql

# 3. Register artifacts
psql $DATABASE_URL -f /opt/singulio-dev/scripts/register-pulsar-artifacts.sql

# 4. Verify
psql $DATABASE_URL -c "SELECT id, display_name, version, status FROM artifacts WHERE id LIKE 'pulsar%';"
```

## Query Artifacts

### List all Pulsar artifacts
```sql
SELECT id, display_name, version, category, status
FROM artifacts
WHERE id LIKE 'pulsar%' OR id LIKE 'tft-%' OR id = 'terminal-wasm';
```

### View with dependencies
```sql
SELECT * FROM artifact_summary
WHERE id IN ('pulsar-daemon', 'pulsar-desktop', 'terminal-wasm');
```

### List endpoints
```sql
SELECT
    a.display_name,
    e.endpoint_type,
    e.method,
    e.path
FROM artifacts a
JOIN artifact_endpoints e ON a.id = e.artifact_id
WHERE a.id = 'pulsar-daemon'
ORDER BY e.endpoint_type;
```

### View dependency tree
```sql
SELECT
    a1.display_name as artifact,
    a2.display_name as depends_on,
    d.dependency_type,
    d.version_constraint
FROM artifact_dependencies d
JOIN artifacts a1 ON d.artifact_id = a1.id
JOIN artifacts a2 ON d.depends_on_artifact_id = a2.id
WHERE a1.id LIKE 'pulsar%';
```

### Get metrics
```sql
SELECT
    metric_name,
    metric_value,
    unit,
    recorded_at
FROM artifact_metrics
WHERE artifact_id = 'pulsar-daemon'
ORDER BY recorded_at DESC;
```

## Use Artifacts via API

### Get artifact details
```bash
curl http://api.singulio.com/v1/artifacts/pulsar-daemon
```

### List artifact versions
```bash
curl http://api.singulio.com/v1/artifacts/pulsar-daemon/versions
```

### Get dependencies
```bash
curl http://api.singulio.com/v1/artifacts/pulsar-daemon/dependencies
```

### Get endpoints
```bash
curl http://api.singulio.com/v1/artifacts/pulsar-daemon/endpoints
```

### Search artifacts
```bash
curl "http://api.singulio.com/v1/artifacts/search?q=terminal&category=infrastructure"
```

## Connect to Pulsar Daemon

### Via gRPC
```bash
# Port 50051
grpcurl -plaintext localhost:50051 list
grpcurl -plaintext localhost:50051 terminal.TerminalService/ListSessions
```

### Via WebSocket
```javascript
const ws = new WebSocket('ws://localhost:3030/ws/<session-id>');
ws.onmessage = (e) => console.log(atob(e.data));
```

### Via IPC
```bash
# Unix socket: /root/.config/orbit/pulsar.sock
orbit terminal ls
```

## Protocol Ports

| Protocol | Port | Type | Purpose |
|----------|------|------|---------|
| IPC | N/A | Unix Socket | Local CLI tools |
| WebSocket | 3030 | TCP | Web browsers |
| gRPC | 50051 | TCP | Service APIs |
| WebTransport | 4433 | UDP | Modern web (HTTP/3) |

## Artifact Metadata Fields

Each artifact includes:

- `id` - Unique identifier
- `name` - Internal name
- `display_name` - Human-readable name
- `description` - Full description
- `category` - Type of artifact
- `version` - Semantic version
- `language` - Primary language
- `repository_url` - Source code location
- `documentation_url` - Docs location
- `status` - Lifecycle status
- `author` - Creator
- `tags[]` - Search tags
- `metadata{}` - Custom JSON metadata

## Example: Complete Artifact Query

```sql
SELECT
    a.id,
    a.display_name,
    a.version,
    a.category,
    a.status,
    a.language,
    a.tags,
    a.metadata->>'protocols' as protocols,
    COUNT(DISTINCT d.depends_on_artifact_id) as dependency_count,
    COUNT(DISTINCT e.id) as endpoint_count,
    COUNT(DISTINCT m.id) as metric_count
FROM artifacts a
LEFT JOIN artifact_dependencies d ON a.id = d.artifact_id
LEFT JOIN artifact_endpoints e ON a.id = e.artifact_id
LEFT JOIN artifact_metrics m ON a.id = m.artifact_id
WHERE a.id = 'pulsar-daemon'
GROUP BY a.id, a.display_name, a.version, a.category, a.status, a.language, a.tags, a.metadata;
```

## Troubleshooting

### Database not available
If PostgreSQL is not running, the registration scripts will fail. Start PostgreSQL first:

```bash
# Docker
docker-compose up -d postgres

# SystemD
sudo systemctl start postgresql

# Manual
pg_ctl start -D /var/lib/postgresql/data
```

### Schema already exists
If you see "already exists" errors, that's OK. The scripts use `CREATE IF NOT EXISTS` and `ON CONFLICT DO UPDATE` to be idempotent.

### Re-register artifacts
To update artifact registration:

```bash
# The script uses ON CONFLICT DO UPDATE, so just re-run:
psql $DATABASE_URL -f /opt/singulio-dev/scripts/register-pulsar-artifacts.sql
```

### View registration logs
The registration script outputs detailed logs:

```sql
-- Should see these messages:
✓ Successfully registered 6 Pulsar terminal system artifacts
✓ Created artifact dependencies and relationships
✓ Registered protocol endpoints (gRPC, WebSocket, WebTransport)
✓ Added version information and metrics
```

## API Specifications

### gRPC API (OpenAPI 3.1)
Location: `/opt/singulio-dev/shared/api-specs/openapi/pulsar-grpc.yaml`

**13 RPC Methods**:
- CreateSession
- ListSessions
- GetSession
- CloseSession
- StreamOutput
- StreamInput
- SendInput
- ResizeTerminal
- SendSignal
- GetHistory
- ClearHistory
- ExportSession
- ImportSession

### WebSocket API (AsyncAPI 3.0)
Location: `/opt/singulio-dev/shared/api-specs/asyncapi/pulsar-websocket.yaml`

**Message Types**:
- TerminalOutput
- TerminalInput
- ControlMessage
- ErrorMessage

**Operations**:
- receiveOutput
- sendInput
- sendControl

## Tags for Discovery

All artifacts are tagged for easy discovery:

**Common Tags**:
- `terminal`
- `daemon`
- `websocket`
- `grpc`
- `webtransport`
- `quic`
- `http3`
- `session-management`
- `pty`
- `wasm`
- `tauri`
- `desktop`

Search by tag:
```sql
SELECT id, display_name FROM artifacts WHERE 'terminal' = ANY(tags);
```

## Related Documentation

- **Architecture**: [WEBTRANSPORT_PHASE_C_COMPLETE.md](./WEBTRANSPORT_PHASE_C_COMPLETE.md)
- **gRPC Phase**: [GRPC_PHASE_A_COMPLETE.md](./GRPC_PHASE_A_COMPLETE.md)
- **WebSocket Phase**: [WEBSOCKET_STREAMING_COMPLETE.md](./WEBSOCKET_STREAMING_COMPLETE.md)
- **Desktop Integration**: [DESKTOP_INTEGRATION_COMPLETE.md](./DESKTOP_INTEGRATION_COMPLETE.md)
- **Registration Status**: [ARTIFACT_REGISTRATION_COMPLETE.md](./ARTIFACT_REGISTRATION_COMPLETE.md)

---

**Quick Reference**: All Pulsar components are registered and ready for production use. Database deployment pending PostgreSQL availability.
