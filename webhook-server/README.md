# Webhook Server

GitHub Webhook server for automated documentation deployment.

## Features

- **Security Verification**
  - HMAC SHA256 signature verification (WEBHOOK_SECRET)
  - GitHub official IP range verification
  - Organization/user whitelist verification
- **Auto Deployment**: Listens to `push` events and automatically:
  - Clones or pulls the repository
  - Installs pip dependencies from `docs/requirements.txt`
  - Runs `make dist` in the `docs/` folder
  - Copies the `dist/` output to the deployment directory
- **Logging**: Records deployment history, queryable via API
- **Health Check**: Provides `/health` endpoint for health monitoring

## Quick Start

### 1. Build

```bash
cd webhook-server
cargo build --release
```

Build artifact location: `target/release/webhook-server`

> **Note**: This project uses `rustls` (pure Rust TLS implementation), no OpenSSL dependency.

### 2. Configure Environment Variables

Create a `.env` file or configure via systemd:

```bash
# Required configuration
WEBHOOK_SECRET=your_webhook_secret_here    # GitHub Webhook secret key

# Directory configuration
SOURCE_DIR=/var/www/docs-source            # Directory to clone repositories (default: /var/www/docs-source)
DEPLOY_DIR=/var/www/docs                   # Directory to deploy dist output (default: /var/www/docs)

# Optional configuration
PORT=5000                              # Service listen port (default: 5000)
ALLOWED_ORGS=your-org                  # Allowed organization whitelist (comma-separated)
ALLOWED_USERS=your-username            # Allowed user whitelist (comma-separated)
SKIP_IP_CHECK=false                    # Whether to skip IP check (default: false)
```

### 3. Run

#### Option 1: Direct execution

```bash
./target/release/webhook-server
```

#### Option 2: Install as systemd service (Recommended for Production)

**Quick Install (Linux):**

```bash
# Make install script executable
chmod +x install.sh

# Run installation (requires root)
sudo ./install.sh
```

**Manual Installation:**

1. **Create installation directory:**

```bash
sudo mkdir -p /opt/webhook-server
sudo cp target/release/webhook-server /opt/webhook-server/
sudo chmod +x /opt/webhook-server/webhook-server
```

2. **Create required directories:**

```bash
sudo mkdir -p /var/www/docs-source
sudo mkdir -p /var/www/docs
sudo chown -R www-data:www-data /opt/webhook-server
sudo chown -R www-data:www-data /var/www/docs-source
sudo chown -R www-data:www-data /var/www/docs
```

3. **Copy and configure systemd service:**

```bash
sudo cp webhook-server.service /etc/systemd/system/
```

4. **Edit `/etc/systemd/system/webhook-server.service`:**

```ini
[Service]
# Required - Set your webhook secret
Environment="WEBHOOK_SECRET=your_secret_key_generate_a_random_one"

# Required - Set allowed users or orgs
Environment="ALLOWED_USERS=crazyjay97"
# Or for organizations:
# Environment="ALLOWED_ORGS=your-org"

# Recommended - Skip IP check when behind Nginx proxy
Environment="SKIP_IP_CHECK=true"

# Optional - Adjust directories if needed
Environment="SOURCE_DIR=/var/www/docs-source"
Environment="DEPLOY_DIR=/var/www/docs"
Environment="PORT=5000"
```

5. **Start the service:**

```bash
sudo systemctl daemon-reload
sudo systemctl enable webhook-server
sudo systemctl start webhook-server
sudo systemctl status webhook-server
```

6. **View logs:**

```bash
# Follow logs in real-time
sudo journalctl -u webhook-server -f

# View recent logs
sudo journalctl -u webhook-server -n 50
```

7. **Manage service:**

```bash
sudo systemctl stop webhook-server    # Stop
sudo systemctl restart webhook-server # Restart
sudo systemctl disable webhook-server # Disable on boot
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/webhook` | POST | GitHub Webhook receiver endpoint |
| `/health` | GET | Health check |
| `/logs` | GET | Get deployment logs |

## Deployment Flow

When a webhook is received for a `push` event to `main` or `master` branch:

1. **Clone/Pull**:
   - If repository doesn't exist: `git clone https://github.com/owner/repo.git` to `SOURCE_DIR/repo`
   - If repository exists: `git pull` to get latest changes

2. **Find docs folder**: Look for `docs/` directory in the cloned repository

3. **Install dependencies**: If `docs/requirements.txt` exists, run `pip install -r requirements.txt`

4. **Build documentation**: Run `make dist` in the `docs/` directory

5. **Deploy**: Copy contents of `docs/dist/` to `DEPLOY_DIR/repo`

## Directory Structure

After deployment, the directory structure will be:

```
/var/www/docs-source/          # SOURCE_DIR
├── repo-a/                    # Cloned repository
│   ├── docs/
│   └── ...

/var/www/docs/                 # DEPLOY_DIR
├── repo-a/                    # Deployed dist contents
│   ├── index.html
│   └── ...
```

## Security Recommendations

1. **Always set WEBHOOK_SECRET**: Prevents unauthorized requests
2. **Configure whitelist**: Restrict organizations/users that can trigger deployment
3. **Skip IP check when behind proxy**: Set `SKIP_IP_CHECK=true` when using Nginx reverse proxy
4. **Use HTTPS**: Use reverse proxy (e.g., Nginx) with HTTPS in production

## Nginx Reverse Proxy Configuration

### 1. Install Nginx

```bash
# Ubuntu/Debian
sudo apt update && sudo apt install nginx

# CentOS/RHEL
sudo yum install nginx
```

### 2. Configure Nginx

Create Nginx configuration file `/etc/nginx/sites-available/webhook-server`:

```nginx
server {
    listen 80;
    server_name your-domain.com;  # Replace with your domain

    # Optional: Redirect HTTP to HTTPS
    # return 301 https://$server_name$request_uri;

    location /github/webhook {
        proxy_pass http://127.0.0.1:5000/webhook;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # GitHub webhook specific settings
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
}
```

### 3. Enable Site

```bash
# Create symlink to enable site
sudo ln -s /etc/nginx/sites-available/webhook-server /etc/nginx/sites-enabled/

# Remove default site if exists
sudo rm /etc/nginx/sites-enabled/default

# Test configuration
sudo nginx -t

# Reload Nginx
sudo systemctl reload nginx
```

### 4. Configure SSL (Recommended for Production)

Using Let's Encrypt with Certbot:

```bash
# Install Certbot
sudo apt install certbot python3-certbot-nginx  # Ubuntu/Debian

# Obtain SSL certificate
sudo certbot --nginx -d your-domain.com
```

### 5. GitHub Webhook Configuration with Nginx

When using Nginx reverse proxy:

1. Go to repository **Settings** > **Webhooks** > **Add webhook**

2. Configure as follows:
   - **Payload URL**: `https://your-domain.com/github/webhook`
   - **Content type**: `application/json`  **(Important: must be JSON, not form)**
   - **Secret**: Same value as `WEBHOOK_SECRET`
   - **Events**: Select **Push events**

3. After adding, GitHub will send a ping event to test the connection

4. Verify the webhook is working - you should see a green checkmark if successful

> **Note**: The Nginx location path `/github/webhook` can be customized, just ensure it proxies to `http://127.0.0.1:5000/webhook`

## Log Viewing

```bash
# View systemd logs
sudo journalctl -u webhook-server -f

# View deployment logs via API
curl http://localhost:5000/logs
```

## Troubleshooting

### 1. Signature Verification Failed

Ensure the Secret in GitHub Webhook configuration exactly matches `WEBHOOK_SECRET`.

### 2. IP Check Failed

If the server is behind a proxy, you may need to set `SKIP_IP_CHECK=true`.

### 3. Deployment Failed - No docs Folder

Ensure your repository has a `docs/` directory with a `Makefile`.

### 4. pip Install Failed

Ensure `pip` is installed and accessible in the system PATH.
