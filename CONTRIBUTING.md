# Contributing to SSH Desktop

Thanks for your interest in contributing! Here's how you can help:

## Development Setup

1. Fork and clone the repo
2. Install dependencies:
   ```bash
   cargo build
   npm install
   ```
3. Start development server:
   ```bash
   npm run dev
   ```

## Pull Requests

1. Fork the repo
2. Create a branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to your branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Bug Reports

Please use the GitHub issue tracker and include:
- Steps to reproduce
- Expected behavior
- Actual behavior
- System info (OS, browser version etc)

## Code Style

- Rust: Use `rustfmt` defaults
- TypeScript/JavaScript: Follow prettier defaults
- Commit messages: Use conventional commits format
