#!/bin/bash

# Script to build documentation and prepare for direct deployment
# Creates a dist/ directory that can be directly served by nginx

set -e  # Exit on any error

echo "Building documentation and preparing for deployment..."

# Remove previous dist directory if it exists
rm -rf dist

# Run build if not already built
if [ ! -d "_build/en/generic/html" ] || [ ! -d "_build/zh_CN/generic/html" ]; then
    echo "Building documentation..."
    source venv/bin/activate && build-docs
    echo "Build completed."
fi

# Create dist directory
mkdir -p dist

# Copy built documentation to dist with language subdirectories
if [ -d "_build/en/generic/html" ]; then
    mkdir -p dist/en
    cp -r _build/en/generic/html/* dist/en/
    echo "✓ Copied English documentation"
fi

if [ -d "_build/zh_CN/generic/html" ]; then
    mkdir -p dist/zh_CN
    cp -r _build/zh_CN/generic/html/* dist/zh_CN/
    echo "✓ Copied Chinese documentation"
fi

# Create a landing page in the root
cat > dist/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Documentation Hub</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            background-color: #f8f9fa;
            color: #333;
        }
        .container {
            text-align: center;
            padding: 3rem 1rem;
        }
        h1 {
            color: #2c3e50;
            margin-bottom: 2rem;
        }
        .language-selector {
            display: flex;
            justify-content: center;
            gap: 1rem;
            margin-top: 2rem;
            flex-wrap: wrap;
        }
        .language-btn {
            display: inline-block;
            padding: 1rem 2rem;
            background-color: #428bca;
            color: white;
            text-decoration: none;
            border-radius: 5px;
            font-size: 1.1rem;
            transition: background-color 0.3s;
        }
        .language-btn:hover {
            background-color: #3071a9;
        }
        .instructions {
            background-color: #fff;
            border: 1px solid #ddd;
            border-radius: 5px;
            padding: 1.5rem;
            margin-top: 2rem;
            text-align: left;
        }
        .instructions h3 {
            margin-top: 0;
        }
        .instructions ul {
            padding-left: 1.5rem;
        }
        .footer {
            margin-top: 3rem;
            color: #6c757d;
            font-size: 0.9rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Project Documentation</h1>
        <p>Welcome to the documentation hub for our project.</p>

        <div class="language-selector">
            <a href="./en/" class="language-btn">English Documentation</a>
            <a href="./zh_CN/" class="language-btn">中文文档</a>
        </div>

        <div class="instructions">
            <h3>Deployment Instructions</h3>
            <p>This documentation must be served through a web server (like nginx) to function properly.</p>
            <ul>
                <li>Directly opening HTML files in a browser will cause "Not Found" errors</li>
                <li>Links and assets require a web server to resolve correctly</li>
                <li>Copy this entire directory to your web server's document root</li>
            </ul>
        </div>

        <div class="footer">
            <p>Documentation built with Sphinx and ESP-IDF docs template</p>
        </div>
    </div>
</body>
</html>
EOF

echo "✓ Created landing page in dist/"

echo ""
echo "🎉 Documentation ready for deployment!"
echo "The dist/ directory contains everything needed for nginx deployment."
echo ""
echo "IMPORTANT: This documentation must be served via a web server."
echo "Opening HTML files directly in a browser will cause navigation issues."
echo ""
echo "To deploy to nginx:"
echo "1. Upload the entire dist/ directory contents to your web server"
echo "2. Configure nginx to serve the directory (see DEPLOYMENT.md)"
echo "3. Access via http://yourdomain.com/ (English) or http://yourdomain.com/zh_CN/ (Chinese)"
echo ""
echo "Directory structure:"
find dist -type d | sort
echo ""
echo "Total files: $(find dist -type f | wc -l)"