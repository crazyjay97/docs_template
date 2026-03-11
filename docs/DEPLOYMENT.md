# Deployment Instructions

The `dist/` directory contains your compiled documentation that is ready to be deployed to a web server.

## Directory Structure

```
dist/
├── index.html                 # Landing page
├── en/                       # English documentation
│   ├── index.html
│   ├── ...
│   └── _static/              # Assets (CSS, JS, images)
└── zh_CN/                    # Chinese documentation
    ├── index.html
    ├── ...
    └── _static/              # Assets (CSS, JS, images)
```

## Important Note About Direct File Access

⚠️ **DO NOT open HTML files directly in your browser** by double-clicking on them. This will result in "Not Found" errors and broken navigation because:

- Sphinx-generated documentation uses relative paths that require a web server to resolve correctly
- AJAX requests and dynamic loading features need proper HTTP headers
- Cross-origin policies restrict direct file access

Always serve the documentation through a web server like nginx, Apache, or a CDN.

## Deploy to Nginx

1. Copy the contents of the `dist/` directory to your nginx web root:
   ```bash
   sudo cp -r dist/* /var/www/html/
   ```

   Or if you're using a subdirectory:
   ```bash
   sudo cp -r dist/* /var/www/html/docs/
   ```

2. Configure your nginx server block to serve static files:
   ```nginx
   server {
       listen 80;
       server_name your-domain.com;

       # Serve documentation at root or subdirectory
       location / {
           root /var/www/html;  # Or /var/www/html/docs if using subdirectory
           index index.html;
           try_files $uri $uri/ @fallback;
       }

       # Handle URLs with trailing slashes for SPA-like behavior
       location @fallback {
           rewrite ^/(.*)$ / permanent;
       }
   }
   ```

3. Reload nginx configuration:
   ```bash
   sudo nginx -t  # Test configuration
   sudo systemctl reload nginx  # Apply changes
   ```

## Alternative Hosting Options

- **GitHub Pages**: Upload the `dist/` directory contents to your GitHub Pages branch
- **Apache Server**: Copy the `dist/` directory contents to Apache's DocumentRoot
- **Cloud Storage**: Upload to S3, GCS, Azure Blob storage as static website
- **CDN**: Serve directly from any static file CDN service

## URLs

After deployment:
- Main page: `http://your-domain.com/`
- English docs: `http://your-domain.com/en/`
- Chinese docs: `http://your-domain.com/zh_CN/`

## Troubleshooting

**Problem**: Clicking links shows "Not Found" error
**Solution**: Make sure you're serving through a web server, not opening files directly in browser

**Problem**: CSS/JS assets not loading
**Solution**: Check nginx configuration for proper static file serving

**Problem**: Links between languages don't work
**Solution**: Verify directory structure matches the expected layout