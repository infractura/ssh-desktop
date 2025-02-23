# SSH Desktop

A collaborative terminal and desktop sharing application that works in your browser.

## Features

- Share terminal sessions securely via web browser
- X11 forwarding for graphical applications
- Real-time collaboration with multiple users
- End-to-end encryption
- Works through firewalls and NAT

## Quick Start

```bash
# Install
cargo install --path crates/sshx

# Start a terminal session
ssh-desktop start

# Start with X11 forwarding
ssh-desktop start --xpra
```

## Development

```bash
# Install dependencies
npm install

# Start dev server
npm run dev
```

See [docs/ssh-desktop.md](docs/ssh-desktop.md) for more details.

## License

MIT
