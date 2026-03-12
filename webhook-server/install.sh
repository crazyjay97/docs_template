#!/bin/bash
# Webhook Server Installation Script
# Usage: sudo ./install.sh

set -e

INSTALL_DIR="/opt/webhook-server"
SERVICE_FILE="webhook-server.service"
BINARY_NAME="webhook-server"

echo "=== Webhook Server Installation ==="

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root (sudo ./install.sh)"
    exit 1
fi

# Create installation directory
echo "Creating installation directory: $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"

# Copy binary
echo "Copying binary to $INSTALL_DIR"
cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Copy service file
echo "Copying service file to /etc/systemd/system/"
cp "$SERVICE_FILE" /etc/systemd/system/

# Create directories
echo "Creating source and deploy directories"
mkdir -p /var/www/docs-source
mkdir -p /var/www/docs

# Set ownership
echo "Setting ownership to www-data"
chown -R www-data:www-data "$INSTALL_DIR"
chown -R www-data:www-data /var/www/docs-source
chown -R www-data:www-data /var/www/docs

# Reload systemd
echo "Reloading systemd daemon"
systemctl daemon-reload

# Enable service
echo "Enabling webhook-server service"
systemctl enable webhook-server

echo ""
echo "=== Installation Complete ==="
echo ""
echo "Next steps:"
echo "1. Edit /etc/systemd/system/webhook-server.service"
echo "   - Set WEBHOOK_SECRET (required)"
echo "   - Set ALLOWED_ORGS or ALLOWED_USERS (required)"
echo "   - Adjust SOURCE_DIR and DEPLOY_DIR if needed"
echo ""
echo "2. Start the service:"
echo "   sudo systemctl start webhook-server"
echo ""
echo "3. Check status:"
echo "   sudo systemctl status webhook-server"
echo ""
echo "4. View logs:"
echo "   sudo journalctl -u webhook-server -f"
