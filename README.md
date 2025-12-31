# shev

Event-driven shell command executor

## Features

- Execute shell commands in response to events
- Timer-based job scheduling
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
SHEV_DB=/path/to/shev.db ./backend
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

# Job inspection
shev job list
shev job show <job-id>

# Trigger events (requires running backend)
shev --url http://127.0.0.1:3000 event trigger my-event

# Reload handlers/timers from database
shev --url http://127.0.0.1:3000 reload
```

### core

Shared library containing database operations and model definitions. Used by both backend and cli.
