# Open Docs Platform

A complete documentation platform based on Sphinx with multi-language support and automated deployment.

## Overview

This project provides:

1. **Sphinx Documentation Template** - A ready-to-use template for technical documentation with internationalization (i18n) support
2. **Webhook Server** - An automated deployment server that listens to GitHub push events and builds documentation

## Project Structure

```
open_docs/
├── docs/                    # Sphinx documentation template
│   ├── en/                  # English documentation source
│   ├── zh_CN/               # Chinese documentation source
│   ├── _static/             # Static assets (CSS, JS, images)
│   ├── conf_common.py       # Common Sphinx configuration
│   ├── Makefile             # Build script
│   └── build_dist.sh        # Distribution build script
│
└── webhook-server/          # Auto-deployment server
    ├── src/
    │   └── main.rs          # Server implementation
    ├── Cargo.toml           # Rust dependencies
    └── README.md            # Server documentation
```

## Quick Start

### Documentation

See [docs/README.md](docs/README.md) for documentation setup and building instructions.

### Webhook Server

See [webhook-server/README.md](webhook-server/README.md) for server setup and configuration.

## Features

### Documentation Platform

- **Multi-language Support**: English and Chinese with easy translation linking
- **Sphinx Theme**: Clean, responsive theme based on sphinx-rtd-theme
- **Chinese Search**: Built-in Chinese text segmentation for accurate search results
- **Professional Layout**: ESP-IDF inspired structure with best practices

### Webhook Server

- **Security**: HMAC SHA256 signature verification, IP whitelist, org/user whitelist
- **Auto Deployment**: Automatic git clone/pull, pip install, make dist, and deployment
- **Retry Mechanism**: Up to 2 retries with 5-minute intervals on failure
- **Logging**: File logging with daily rotation and configurable paths
- **Health Check**: `/health` endpoint for monitoring
- **API**: `/logs` endpoint to query deployment history

## Architecture

```
GitHub Push Event
       │
       ▼
┌─────────────────────┐
│   Webhook Server    │
│   (port 5000)       │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│  Git Clone/Pull     │
│  pip install        │
│  make clean         │
│  make dist          │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│   Deploy to         │
│   /var/www/docs     │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│   Nginx serves      │
│   static files      │
└─────────────────────┘
```

## Environment Variables

### Webhook Server Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `WEBHOOK_SECRET` | - | GitHub Webhook secret (required for security) |
| `PORT` | 5000 | Server listen port |
| `SOURCE_DIR` | /var/www/docs-source | Directory to clone repositories |
| `DEPLOY_DIR` | /var/www/docs | Directory to deploy built documentation |
| `ALLOWED_ORGS` | - | Comma-separated list of allowed organizations |
| `ALLOWED_USERS` | - | Comma-separated list of allowed users |
| `SKIP_IP_CHECK` | false | Skip GitHub IP verification (use behind proxy) |
| `LOG_FILE_PATH` | webhook-server.log | Path to log file |

## License

MIT License
