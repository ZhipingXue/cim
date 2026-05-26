# CIM App Group — Agent Guide

## Overview

This project implements a semiconductor equipment CIM (Computer Integrated Manufacturing) app group following SEMI standards:

- **E40** — Process Job Management (PJob)
- **E94** — Control Job Management (CJob)
- **E87** — Carrier Management (CMS)
- **E90** — Substrate Tracking
- **E116** — Alarm / Event / State Machine (distributed, not standalone)
- **E125** — Equipment Metadata (reserved)
- **E134** — Data Collection Plan

## Architecture

```
cim/
├── proto/           # Shared gRPC proto definitions
├── rust/            # Rust workspace (semi-standard apps, data collection, registry)
├── dotnet/          # .NET 8.0 solution (control apps)
├── scripts/         # Build & deploy scripts
└── docs/            # Architecture documentation
```

### Communication

All apps communicate via **gRPC** using shared proto definitions. Service discovery is handled by a central **Service Registry** (port 50000).

Data collection uses **Zenohd** as the pub/sub/query core.

## Project Structure

### Proto (`/proto/`)

| File | Purpose |
|------|---------|
| `cim_common.proto` | Shared types (ModuleId, SubstrateId, Alarm, Event, StatusCode) |
| `e40_process_job.proto` | E40 PJob CRUD, state machine, commands |
| `e94_control_job.proto` | E94 CJob CRUD, PJob grouping, execution control |
| `e87_carrier_management.proto` | E87 LoadPort / Carrier states, slot map |
| `e90_substrate_tracking.proto` | E90 Substrate location, history, batch |
| `e116_alarm_event.proto` | E116 Alarm reporting, event publishing, state transitions |
| `e125_metadata.proto` | E125 Equipment metadata, capabilities |
| `e134_data_collection.proto` | E134 DCP dynamic object with state, reports |
| `control_app.proto` | Generic control app interface |
| `service_registry.proto` | Central registry (register / heartbeat / discover) |

### Rust Workspace (`/rust/`)

| Crate | Type | Purpose |
|-------|------|---------|
| `semi_common` | lib | Shared traits, state machine, types, errors |
| `service_registry` | bin | Central service registry (port 50000) |
| `e40_process_job` | lib + bin | E40 PJob service (port 50041) |
| `e94_control_job` | lib + bin | E94 CJob service (port 50042) |
| `e87_carrier_manager` | lib + bin | E87 CMS service (port 50043) |
| `e90_substrate_tracker` | lib + bin | E90 tracking service (port 50044) |
| `e125_metadata_manager` | lib + bin | E125 metadata service (port 50045) |
| `data_collection` | bin | Zenohd wrapper + E134 DCP service (port 50046) |

### .NET Solution (`/dotnet/`)

| Project | Type | Purpose |
|---------|------|---------|
| `ControlApp.Common` | lib | Shared interfaces, base classes, state control |
| `ChamberControl` | exe | Chamber control app (port 50100+) |
| `RobotControl` | exe | Robot control app (port 50100+) |
| `SubstrateCacheControl` | exe | Substrate cache control app (port 50100+) |
| `LoadPortControl` | exe | LoadPort control app (port 50100+) |

## Default Ports

| Service | Port |
|---------|------|
| Service Registry | 50000 |
| E40 Process Job | 50041 |
| E94 Control Job | 50042 |
| E87 Carrier Manager | 50043 |
| E90 Substrate Tracker | 50044 |
| E125 Metadata Manager | 50045 |
| E134 Data Collection | 50046 |
| Control Apps | 50100 + instance offset |

## Build Requirements

- **Rust**: 1.94.0+ (stable)
- **.NET SDK**: 8.0+
- **protoc**: 3.21.12 (at `D:\protoc-21.12-win64\bin\protoc.exe`)
- **Zenohd**: installed via `cargo binstall zenohd`

## Build Steps

### 1. Set PROTOC environment variable

```powershell
$env:PROTOC = "D:\protoc-21.12-win64\bin\protoc.exe"
```

### 2. Build Rust workspace

```powershell
cd rust
cargo build --workspace
```

### 3. Build .NET solution

```powershell
cd dotnet
dotnet build CimControlApps.sln
```

## Running

### 1. Install Zenohd (first time)

```powershell
.\scripts\install-zenohd.ps1
```

### 2. Start services (development)

```powershell
.\scripts\run-all.ps1
```

Or manually:

```powershell
# Terminal 1: Zenohd
zenohd

# Terminal 2: Service Registry
cd rust
cargo run -p service_registry

# Terminal 3: E40 Process Job
cargo run -p e40_process_job

# Terminal 4: E94 Control Job
cargo run -p e94_control_job

# ... etc
```

## Configuration

Each app loads `config.json` from its working directory:

```json
{
  "service_id": "e40-process-job-01",
  "service_type": "e40_process_job",
  "grpc_host": "0.0.0.0",
  "grpc_port": 50041,
  "registry_address": "http://localhost:50000",
  "heartbeat_interval_sec": 30,
  "log_level": "info",
  "zenohd_url": "tcp/localhost:7447"
}
```

## Naming Conventions

| Layer | Convention |
|-------|-----------|
| Proto packages | `cim.common`, `cim.e40`, `cim.e94`, ... |
| Rust crates | `snake_case` (e.g., `semi_common`, `e40_process_job`) |
| Rust modules | `snake_case` |
| .NET namespaces | `PascalCase` (e.g., `Cim.ControlApp.Common`) |
| .NET classes | `PascalCase` |
| Config files | `config.json` per app |

## State Machine Framework

The Rust `semi_common` crate provides a generic state machine:

```rust
use semi_common::state_machine::StateMachine;

let sm = StateMachine::new(
    "pjob-001".to_string(),
    PJobState::Queued,
    transitions,  // HashMap<State, HashSet<State>>
    context,      // Custom context
);

sm.transition(PJobState::Executing).await?;
```

## E116 Integration (Distributed)

E116 is **not a standalone app**. Each app embeds:

- `AlarmReporter` trait — report/clear alarms
- `EventPublisher` trait — publish events
- `StateMachineReporter` trait — report state transitions

Reports are sent to the E116 endpoint in the CIM Host process.

## Data Collection (E134)

Data Collection Plans are **dynamic objects with state**:

```
CREATED --(activate)--> ACTIVATED --(pause)--> PAUSED --(resume)--> ACTIVATED
   |                      |                        |
   |                   (deactivate)             (deactivate)
   |                      |                        |
   v                      v                        v
 DELETED <------------ DEACTIVATED <-------------+
```

Parameters can be added/removed in `CREATED` or `DEACTIVATED` state.

## Service Discovery

Apps use the central Service Registry:

1. **Register** on startup
2. **Heartbeat** every 30 seconds
3. **Discover** other services by type
4. **Watch** for real-time service changes

Services are removed after 3 missed heartbeats.

## Logging

- **Rust**: `tracing` crate with structured logging
- **.NET**: `Microsoft.Extensions.Logging`

Set `log_level` in `config.json`: `trace`, `debug`, `info`, `warn`, `error`.

## Key Dependencies

### Rust
- `tonic` 0.14.5 — gRPC
- `prost` 0.14.3 — protobuf
- `tokio` 1.52.2 — async runtime
- `serde` 1.0.228 — serialization
- `thiserror` 2.0.18 — error derives
- `chrono` 0.4.44 — timestamps
- `tracing` 0.1.44 — logging
- `zenoh` 1.9.0 — pub/sub

### .NET
- `Grpc.AspNetCore` — gRPC server
- `Grpc.Net.Client` — gRPC client
- `Google.Protobuf` — protobuf
- `Microsoft.Extensions.Hosting.WindowsServices` — Windows service hosting

## Notes for Agents

- Do **not** modify the CIM Host process (already designed externally).
- Do **not** create a standalone E116 app (distributed integration only).
- E125 is reserved — implement minimal placeholder.
- Always use JSON config files per app.
- Follow the proto package naming: `cim.{domain}`.
- When adding new apps, register default ports in this guide.
