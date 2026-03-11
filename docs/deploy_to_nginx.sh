#!/bin/bash

# Script to prepare documentation for nginx deployment
# This script creates a directory structure suitable for nginx serving

set -e  # Exit on any error

echo "Preparing documentation for nginx deployment..."

# Remove previous deployment directory if it exists
rm -rf nginx_deploy

# Create deployment directory
mkdir -p nginx_deploy

# Copy built documentation to appropriate language subdirectories
if [ -d "_build/en/generic/html" ]; then
    mkdir -p nginx_deploy/en
    cp -r _build/en/generic/html/* nginx_deploy/en/
    echo "Copied English documentation to nginx_deploy/en/"
fi

if [ -d "_build/zh_CN/generic/html" ]; then
    mkdir -p nginx_deploy/zh_CN
    cp -r _build/zh_CN/generic/html/* nginx_deploy/zh_CN/
    echo "Copied Chinese documentation to nginx_deploy/zh_CN/"
fi

# Create a simple index.html to redirect to default language (English)
cat > nginx_deploy/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Documentation</title>
    <script>
        // Redirect to English documentation by default
        window.location.href = "./en/";
    </script>
</head>
<body>
    <p>If you are not redirected automatically, <a href="./en/">click here</a> to go to the English documentation.</p>
    <p><a href="./zh_CN/">中文文档</a></p>
</body>
</html>
EOF

echo "Documentation prepared for nginx deployment in nginx_deploy/ directory."
echo ""
echo "To deploy, simply copy the nginx_deploy directory contents to your nginx web root."
echo "For example: sudo cp -r nginx_deploy/* /var/www/html/"
echo ""
echo "Directory structure:"
ls -la nginx_deploy/