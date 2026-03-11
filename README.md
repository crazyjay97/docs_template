# Open Docs - Production-Grade Documentation Platform

A robust, scalable documentation platform built with Rust, featuring:

- **Automatic builds** on Git pushes via webhooks
- **Diff-based builds** that only rebuild changed documentation
- **Atomic deployments** using symlinks
- **Multi-version support** with easy switching
- **Build queue** with configurable workers
- **Email notifications** for build failures
- **Prometheus metrics** for monitoring
- **High-performance static serving** via Nginx

## Architecture

```
                     ┌──────────────┐
                     │    Users     │
                     └──────┬───────┘
                            │
                            ▼
                      ┌────────────┐
                      │   Nginx    │
                      │ Static CDN │
                      └─────┬──────┘
                            │
               ┌────────────┴────────────┐
               ▼                         ▼
        Static Docs               Rust Admin API
       (/data/docs)              (Control Plane)
                                        │
                                        ▼
                               Build Scheduler
                                        │
                        ┌───────────────┼───────────────┐
                        ▼               ▼               ▼
                   Build Worker    Build Worker    Build Worker
                        │               │               │
                        ▼               ▼               ▼
                      Git Clone / Fetch / Diff
                        │
                        ▼
                      Sphinx Build
                        │
                        ▼
                  Artifact Publish
                        │
                        ▼
                    Version Store
```

## Installation

### Requirements

- Rust 1.70+ (edition 2021)
- Python 3.8+ with Sphinx
- Nginx (for static serving)
- Git

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run -- --config config.toml
```

Or run without configuration (uses defaults):

```bash
cargo run
```

## Configuration

The configuration is in TOML format.

See `config.toml` for configuration options.

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `api_port` | Port for the API server | `8080` |
| `worker_count` | Number of build workers | `3` |
| `queue_capacity` | Maximum jobs in queue | `100` |
| `git_base_path` | Base path for Git repos | `/srv/docsys/repos` |
| `repos_path` | Path to clone repositories | `/srv/docsys/repos` |
| `build_base_path` | Path for temporary builds | `/srv/docsys/build` |
| `artifacts_path` | Path for published artifacts | `/srv/docsys/artifacts` |
| `logs_path` | Path for build logs | `/srv/docsys/logs` |
| `sphinx_python_path` | Path to Python for Sphinx | `python3` |
| `sphinx_build_cmd` | Sphinx build command | `sphinx-build -b html -j auto` |
| `nginx_root` | Path for Nginx to serve | `/data/docs` |
| `email_smtp` | Email SMTP configuration | `None` |

## API Endpoints

### Git Webhook

```
POST /webhook/git
```

Payload:

```json
{
  "repository": {
    "name": "project-name",
    "url": "https://github.com/user/repo.git"
  },
  "ref": "refs/heads/main",
  "after": "commit-hash"
}
```

### Manual Build

```
POST /admin/build
```

Payload:

```json
{
  "project": "project-name",
  "repo": "https://github.com/user/repo.git",
  "branch": "main",
  "commit": "commit-hash"
}
```

### Queue Status

```
GET /admin/queue
```

Response:

```json
{
  "queue_size": 5,
  "workers": 3
}
```

## Directory Structure

```
/srv/docsys
├── repos/          # Cloned repositories
├── build/          # Temporary build artifacts
├── artifacts/      # Published documentation versions
└── logs/           # Build logs

/data/docs          # symlinks to latest artifacts for nginx
```

## Deployment

1. Install Nginx and configure it to serve `/data/docs/*`
2. Set up the Rust application with appropriate paths
3. Configure Git webhook to point to `/webhook/git`
4. Monitor logs

## Notifications

The system sends email notifications on build failures.

## Monitoring

Metrics are exposed via the `metrics` crate for integration with Prometheus.

## Scaling

To scale horizontally:

1. Add more workers (increase `worker_count`)
2. Add more Nginx instances behind load balancer

## Contributing

Contributions are welcome!
