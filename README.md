# shev

Event-driven shell command executor

## Installation

Using [just](https://github.com/casey/just):

```sh
just install
```

Or manually with cargo:

```sh
cargo install --path backend
cargo install --path cli
```

## Features

- Execute shell commands in response to events
- Timer-based recurring jobs (interval-based)
- Schedule-based jobs (UTC time-based, one-shot or daily)
- Job management via HTTP API

## Projects

### backend

The main server that processes events and executes shell commands.

- Listens for HTTP events on configurable port
- Manages event handlers and timer schedules
- Executes shell commands with timeout support
- Persists state in SQLite database

```sh
# Database path defaults to shev.db next to executable
# Override with SHEV_DB environment variable
SHEV_DB=/path/to/shev.db shev-backend

# Listen on localhost only
shev-backend 

# Listen on all interfaces
shev-backend --listen

# Allow specific IPs for read access (GET requests)
shev-backend --listen --allow 192.168.1.100 --allow 10.0.0.50

# Allow specific IPs for write access (POST/PUT/DELETE)
# Write IPs also get read access
shev-backend --listen --allow-write 192.168.1.100
```

### cli

Command-line tool for managing handlers, timers, jobs, and triggering events.

```sh
# Set database path (required)
export SHEV_DB=/path/to/shev.db

# Handler management
shev handler add my-event bash "echo hello"
shev handler list
shev handler delete my-event

# Timer management
shev timer add my-timer 60 --context "optional context"
shev timer list

# Schedule management (UTC time-based)
shev schedule add my-schedule "2025-01-15T14:30:00Z" --context "optional context"
shev schedule add daily-task "2025-01-15T09:00:00Z" --periodic
shev schedule list

# Job inspection
shev job list
shev job show <job-id>
shev job cancel <job-id>

# Trigger events (requires running backend)
shev --url http://127.0.0.1:3000 event trigger my-event

# Reload handlers/timers from database
shev --url http://127.0.0.1:3000 reload
```

### core

Shared library containing database operations and model definitions. Used by both backend and cli.

### ui

Web dashboard for managing and monitoring shev. Built with SvelteKit.

```sh
cd ui
npm install
npm run dev
```
