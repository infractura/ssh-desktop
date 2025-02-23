# SSH Desktop

## Architecture

SSH Desktop integrates with Xpra to provide secure X11 forwarding over WebSocket connections, enabling browser-based access to graphical applications with collaborative features.

### Components

1. **Core**
   - Manages terminal sessions and connections
   - Handles authentication and end-to-end encryption
   - Routes WebSocket connections
   - Provides collaborative features with cursor sharing
   - Manages infinite canvas for window arrangement

2. **Xpra Integration**
   - Dynamic display pool (100-599)
   - WebSocket ports starting at 14500
   - Configurable window manager support
   - HTML5 client with WebSocket forwarding
   - Automatic process lifecycle management
   - Session monitoring and metrics collection

3. **Display Management**
   - Thread-safe display allocation
   - Automatic port assignment
   - Process monitoring and cleanup
   - Resource usage tracking
   - Idle session termination

## Implementation Details

### Display Management

The display management system consists of two key components:

1. **Display Pool**
```rust
pub struct DisplayPool {
    used_displays: Arc<Mutex<HashSet<u16>>>,
}
```
- Thread-safe display number allocation (100-599)
- Automatic cleanup on session end
- Concurrent session support
- Display number reuse

2. **Xpra Display**
```rust
pub struct XpraDisplay {
    display: u16,
    process: Child,
    websocket_port: u16,
}
```
- Process lifecycle management
- WebSocket port allocation (14500 + display)
- Port availability checking
- Automatic process termination

### Metrics and Monitoring

```rust
pub struct XpraMetrics {
    total_sessions: AtomicU64,
    active_sessions: AtomicU64,
    failed_sessions: AtomicU64,
    idle_terminations: AtomicU64,
    start_time: Instant,
}
```
- Lock-free atomic counters
- Session lifecycle tracking
- Failure monitoring
- Uptime tracking
- JSON-serializable snapshots

### Runner Integration

```rust
pub enum Runner {
    Shell(String),
    Xpra {
        display: u16,
        wm: String,
    },
    Echo,
}
```
- Unified terminal/X11 interface
- Custom window manager support
- Test mode with Echo runner
- Process isolation per session

### Process Flow

1. Client requests X11 session via SSH Desktop
2. Display allocation:
   - Get available display from pool (100-599)
   - Assign WebSocket port (14500 + display)
   - Verify port availability

3. Xpra process initialization:
   - Start with allocated display
   - Configure WebSocket binding
   - Launch specified window manager
   - Enable HTML5 client support
   - Disable audio (pulseaudio=no)
   - Set non-daemon mode
   - Enable exit with children

4. Session monitoring:
   - Track process status
   - Update metrics (active sessions, etc)
   - Monitor for failures
   - Handle idle timeouts

5. Cleanup on session end:
   - Kill Xpra process
   - Release display number
   - Update metrics
   - Clean up resources

## Security Considerations

- End-to-end encryption using Argon2 and AES
- Process isolation:
  - Separate display numbers
  - Unique WebSocket ports
  - Individual process spaces

- Resource protection:
  - Display number limits (max 500)
  - Port availability checking
  - Process monitoring
  - Automatic cleanup

- Session management:
  - Idle session termination
  - Process lifecycle tracking
  - Failed session handling
  - Metrics collection

- Access control:
  - Optional read-only mode
  - Session-specific URLs
  - Process ownership isolation

## Future Enhancements

1. Performance Optimizations
   - WebSocket compression
   - Adaptive frame rates
   - Smart buffer management
   - Connection quality metrics

2. Advanced Features
   - Clipboard synchronization
   - File transfer support
   - Audio forwarding
   - Multi-monitor support
   - Session recording/playback

3. Management Features
   - Resource usage limits
   - Session quotas
   - Admin dashboard
   - Usage analytics
   - Performance monitoring

4. Integration Improvements
   - More window managers
   - GPU acceleration
   - Custom display resolutions
   - Touch input support
   - IME support

## Configuration

### Prerequisites

- Xpra 4.0+ installed
- A supported window manager (gnome-flashback, xfce4, etc)
- Rust 1.70+ for building from source
- System with X11 support

### Installation

1. **From Source**
   ```bash
   cargo install --path crates/sshx
   ```

2. **Binary Release**
   Download from GitHub releases and install:
   ```bash
   sudo install -m 755 ssh-desktop /usr/local/bin/
   ```

### System Service

```ini
[Unit]
Description=SSH Desktop Xpra Service
After=network.target
Documentation=https://github.com/user/ssh-desktop

[Service]
ExecStart=/usr/bin/ssh-desktop xpra-service
Environment=RUST_LOG=info
Environment=SSH_DESKTOP_SERVER=localhost
Environment=DISPLAY_MIN=100
Environment=DISPLAY_MAX=599
Type=notify
Restart=always
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

### Environment Variables

- `RUST_LOG`: Logging level (error, warn, info, debug, trace)
- `SSH_DESKTOP_SERVER`: Server URL for connection (default: localhost)
- `DISPLAY_MIN`: Minimum display number (default: 100)
- `DISPLAY_MAX`: Maximum display number (default: 599)
- `SHELL`: Default shell for terminal sessions
- `WM`: Window manager to use (default: gnome-flashback)

### Metrics and Monitoring

Available metrics through the API:
- Total sessions
- Active sessions
- Failed sessions
- Idle terminations
- System uptime
- Resource usage

### Logging and Analysis

The system includes comprehensive logging and analysis capabilities:

1. **Log Analysis**
```rust
Command::Analyze {
    days: i64,        // Analysis period
    format: String,   // Output format (text/json)
}
```
- Historical session analysis
- Configurable time periods
- Multiple output formats
- Performance trending

2. **Status Monitoring**
```rust
Command::Status {
    format: String,    // Output format
    active_only: bool, // Filter to active sessions
}
```
- Real-time session status
- Filtered view options
- Format customization
- Active session tracking

3. **Log Management**
- Automatic log rotation
- Structured logging
- Performance metrics
- Error tracking
- Session correlation

4. **Visualization**
- Session statistics
- Usage patterns
- Error rates
- Resource utilization
- Performance trends

## Troubleshooting

### Common Issues

1. **Display Allocation Failures**
   - Error: "No available display numbers"
   - Cause: All displays in range 100-599 are in use
   - Solution: Increase DISPLAY_MAX or cleanup zombie sessions

2. **Port Conflicts**
   - Error: "Address already in use"
   - Cause: WebSocket port already taken
   - Solution: Check for other services using ports 14500-15000

3. **Process Management**
   - Error: "Failed to kill Xpra process"
   - Cause: Process cleanup issues
   - Solution: Check process ownership and permissions

4. **Session Issues**
   - Error: "Session failed to start"
   - Cause: Window manager or Xpra configuration
   - Solution: Verify Xpra installation and WM availability

### Debugging

1. **Logging Levels**
   Set `RUST_LOG` environment variable:
   - error: Only errors
   - warn: Warnings and errors
   - info: Normal operation logs
   - debug: Detailed debugging
   - trace: Full protocol traces

2. **Metrics Analysis**
   Use the status command with JSON output:
   ```bash
   ssh-desktop status --format json
   ```

3. **Log Analysis**
   Analyze recent issues:
   ```bash
   ssh-desktop analyze --days 1 --format json
   ```

## Resources

- [Project Repository](https://github.com/user/ssh-desktop)
- [Issue Tracker](https://github.com/user/ssh-desktop/issues)
- [Xpra Documentation](https://xpra.org/docs/)
- [Contributing Guide](CONTRIBUTING.md)
