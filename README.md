# shev

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

Command-line tool for managing handlers, timers, jobs, and triggering events via HTTP API.

```sh
# Server URL defaults to http://127.0.0.1:3000
# Override with --url flag or SHEV_URL environment variable
export SHEV_URL=http://192.168.1.100:3000

# Handler management
shev handler add my-event -s bash -c "echo hello"
shev handler add my-event -s pwsh -c "Write-Host 'hello'" -t 30 -e "KEY=value"
shev handler list
shev handler show my-event
shev handler update my-event -c "echo updated"
shev handler remove my-event

# Timer management (interval-based recurring events)
shev timer add my-timer -i 60 -c "optional context"
shev timer list
shev timer show my-timer
shev timer update my-timer -i 120
shev timer remove my-timer

# Schedule management (UTC time-based)
shev schedule add my-schedule -t "2025-01-15T14:30:00Z" -c "optional context"
shev schedule add daily-task -t "2025-01-15T09:00:00Z" -p  # periodic (daily)
shev schedule list
shev schedule show my-schedule
shev schedule update my-schedule -t "2025-01-15T15:00:00Z"
shev schedule remove my-schedule

# Job inspection
shev job list
shev job list -s failed -l 10           # filter by status, limit results
shev job show <job-id>
shev job show <job-id> -n 0             # show full output (no line limit)
shev job cancel <job-id>

# Trigger events
shev event trigger my-event
shev event trigger my-event -c "context data"

# Configuration
shev config show
shev config set port 3001               # requires restart
shev config set queue_size 200          # requires restart

# Reload handlers/timers/schedules from database
shev reload
```

### core

Shared library containing database operations and model definitions. Used by both backend and cli.
