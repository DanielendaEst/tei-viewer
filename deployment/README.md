# TEI Viewer - Deployment Guide

This guide covers deploying the TEI Viewer to various hosting platforms and web servers.

## Table of Contents

- [Quick Start](#quick-start)
- [Building for Production](#building-for-production)
- [Deployment Options](#deployment-options)
  - [Option 1: Docker (Easiest)](#option-1-docker-easiest)
  - [Option 2: Nginx (Recommended for VPS)](#option-2-nginx-recommended-for-vps)
  - [Option 3: Apache](#option-3-apache)
  - [Option 4: Cloud Platforms](#option-4-cloud-platforms)
  - [Option 5: Simple Python Server (Testing)](#option-5-simple-python-server-testing)
- [Important Configuration](#important-configuration)
- [Troubleshooting](#troubleshooting)

---

## Quick Start

### Using Docker (Recommended for Quick Deployment)

```bash
# Build and run with Docker Compose
docker-compose up -d

# Access at http://localhost:8080
```

### Manual Deployment

1. **Build the production bundle:**
   ```bash
   ./deploy.sh
   ```

2. **Deploy the `dist/` folder** to your web server

3. **Configure your server** to serve static files and handle WASM properly

---

## Building for Production

### Prerequisites

- Rust and Cargo installed
- Trunk build tool: `cargo install trunk`
- WASM target: `rustup target add wasm32-unknown-unknown`

### Build Command

```bash
# From the tei-viewer directory
./deploy.sh
```

Or manually:

```bash
# Clean build
rm -rf dist

# Ensure projects are in public/
mkdir -p public
cp -r projects public/

# Build optimized release
trunk build --release
```

The output will be in the `dist/` directory, ready for deployment.

---

## Deployment Options

### Option 1: Docker (Easiest)

**Why Docker?** Self-contained, consistent across environments, easy to deploy and scale.

#### Quick Start with Docker Compose

```bash
# Build and start the container
docker-compose up -d

# View logs
docker-compose logs -f

# Stop the container
docker-compose down
```

The application will be available at `http://localhost:8080`

#### Manual Docker Build

```bash
# Build the image
docker build -t tei-viewer .

# Run the container
docker run -d \
  --name tei-viewer \
  -p 8080:80 \
  --restart unless-stopped \
  tei-viewer

# View logs
docker logs -f tei-viewer

# Stop and remove
docker stop tei-viewer
docker rm tei-viewer
```

#### Production Deployment with Docker

For production, you can:

1. **Push to Docker Hub:**
   ```bash
   docker tag tei-viewer yourusername/tei-viewer:latest
   docker push yourusername/tei-viewer:latest
   ```

2. **Deploy on your server:**
   ```bash
   # On the server
   docker pull yourusername/tei-viewer:latest
   docker run -d -p 80:80 --restart unless-stopped yourusername/tei-viewer:latest
   ```

3. **Or use Docker Compose on the server:**
   ```bash
   # Copy docker-compose.yml to server
   scp docker-compose.yml user@server:/opt/tei-viewer/
   
   # On the server
   cd /opt/tei-viewer
   docker-compose up -d
   ```

#### Adding SSL with Docker + Traefik

Uncomment the Traefik section in `docker-compose.yml` and configure your domain for automatic SSL certificates.

---

### Option 2: Nginx (Recommended for VPS)

**Why Nginx?** Fast, lightweight, excellent for static files, easy SSL setup.

#### Step 1: Install Nginx

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install nginx

# CentOS/RHEL
sudo yum install nginx
```

#### Step 2: Upload Your Files

```bash
# Create deployment directory
sudo mkdir -p /var/www/tei-viewer

# Upload the dist folder
sudo cp -r dist /var/www/tei-viewer/

# Set permissions
sudo chown -R www-data:www-data /var/www/tei-viewer
sudo chmod -R 755 /var/www/tei-viewer
```

#### Step 3: Configure Nginx

```bash
# Copy the config file
sudo cp deployment/nginx.conf /etc/nginx/sites-available/tei-viewer

# Edit the config to set your domain name
sudo nano /etc/nginx/sites-available/tei-viewer
# Change: server_name tei-viewer.example.com;

# Enable the site
sudo ln -s /etc/nginx/sites-available/tei-viewer /etc/nginx/sites-enabled/

# Test configuration
sudo nginx -t

# Restart Nginx
sudo systemctl restart nginx
```

#### Step 4: Configure Firewall

```bash
sudo ufw allow 'Nginx Full'
sudo ufw enable
```

#### Step 5: Add SSL (Recommended)

```bash
# Install certbot
sudo apt install certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d tei-viewer.example.com

# Certbot will automatically configure HTTPS
```

---

### Option 3: Apache

**Why Apache?** Widely supported, good for shared hosting, familiar .htaccess support.

#### Step 1: Install Apache

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install apache2

# CentOS/RHEL
sudo yum install httpd
```

#### Step 2: Enable Required Modules

```bash
sudo a2enmod rewrite
sudo a2enmod headers
sudo a2enmod expires
sudo a2enmod deflate
sudo a2enmod ssl
```

#### Step 3: Upload Your Files

```bash
# Create deployment directory
sudo mkdir -p /var/www/tei-viewer

# Upload the dist folder
sudo cp -r dist /var/www/tei-viewer/

# Set permissions
sudo chown -R www-data:www-data /var/www/tei-viewer
sudo chmod -R 755 /var/www/tei-viewer
```

#### Step 4: Configure Apache

```bash
# Copy the config file
sudo cp deployment/apache.conf /etc/apache2/sites-available/tei-viewer.conf

# Edit the config
sudo nano /etc/apache2/sites-available/tei-viewer.conf
# Change: ServerName tei-viewer.example.com

# Enable the site
sudo a2ensite tei-viewer

# Test configuration
sudo apache2ctl configtest

# Restart Apache
sudo systemctl restart apache2
```

#### Step 5: Add SSL

```bash
# Install certbot
sudo apt install certbot python3-certbot-apache

# Get certificate
sudo certbot --apache -d tei-viewer.example.com
```

---

### Option 4: Cloud Platforms

#### Netlify (Easy, Free Tier)

1. **Build locally:**
   ```bash
   ./deploy.sh
   ```

2. **Deploy via Netlify CLI:**
   ```bash
   npm install -g netlify-cli
   netlify deploy --dir=dist --prod
   ```

   Or use the web UI: drag and drop the `dist/` folder at https://app.netlify.com/drop

3. **Configure:**
   - Add `_redirects` file to `dist/` for SPA routing:
     ```
     /*    /index.html   200
     ```

#### Vercel

1. **Install Vercel CLI:**
   ```bash
   npm install -g vercel
   ```

2. **Deploy:**
   ```bash
   ./deploy.sh
   cd dist
   vercel --prod
   ```

#### GitHub Pages

1. **Build:**
   ```bash
   ./deploy.sh
   ```

2. **Push to gh-pages branch:**
   ```bash
   git checkout --orphan gh-pages
   git rm -rf .
   cp -r dist/* .
   git add .
   git commit -m "Deploy to GitHub Pages"
   git push origin gh-pages --force
   ```

3. **Enable in Settings:**
   - Go to repository Settings → Pages
   - Select `gh-pages` branch

#### AWS S3 + CloudFront

1. **Build:**
   ```bash
   ./deploy.sh
   ```

2. **Upload to S3:**
   ```bash
   aws s3 sync dist/ s3://your-bucket-name/ --delete
   ```

3. **Configure S3 bucket:**
   - Enable static website hosting
   - Set index document to `index.html`
   - Set error document to `index.html` (for SPA routing)

4. **Set up CloudFront** for CDN and HTTPS

---

### Option 5: Simple Python Server (Testing Only)

**⚠️ NOT for production!** Use this only for local testing.

```bash
# After building
./deploy.sh

# Serve on port 8000
python3 -m http.server -d dist 8000

# Open browser to http://localhost:8000
```

---

## Important Configuration

### WASM MIME Type (CRITICAL!)

Your server **must** serve `.wasm` files with the correct MIME type:

```
application/wasm
```

This is already configured in the provided `nginx.conf` and `apache.conf` files.

**To verify:**
```bash
curl -I https://your-domain.com/your-app.wasm
# Should show: Content-Type: application/wasm
```

### CORS Headers for WASM

Some browsers require these headers for WASM:

```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

Already included in provided configs.

### SPA Routing

Since this is a Single Page Application, configure your server to redirect all routes to `index.html`:

- **Nginx:** `try_files $uri $uri/ /index.html;`
- **Apache:** Use rewrite rules (see `apache.conf`)
- **Netlify/Vercel:** Use `_redirects` or `vercel.json`

---

## Troubleshooting

### Problem: Blank page after deployment

**Solutions:**
1. Check browser console for errors
2. Verify WASM MIME type is set correctly
3. Check that all files uploaded (especially `.wasm` files)
4. Verify CORS headers if needed

### Problem: "Failed to fetch" errors

**Solutions:**
1. Check file paths are correct (case-sensitive on Linux!)
2. Verify `projects/` directory is in the right location
3. Check server logs for 404 errors

### Problem: Routes don't work (404 on refresh)

**Solution:**
Configure SPA routing (see above). All routes should return `index.html`.

### Problem: WASM fails to load

**Solutions:**
1. Check MIME type: `curl -I https://your-domain.com/file.wasm`
2. Verify CORS headers are set
3. Check browser console for specific error
4. Ensure `.wasm` files aren't being compressed incorrectly

### Problem: Images don't load

**Solutions:**
1. Verify `public/projects/` structure is preserved in `dist/`
2. Check image paths in TEI files are relative
3. Verify `projects/` directory was copied before build

---

## Performance Optimization

### Enable Compression

Both provided configs enable gzip compression for:
- HTML, CSS, JavaScript
- JSON, XML
- WASM files

### Enable Caching

Static assets are cached for 1 year:
- WASM files
- JavaScript bundles
- CSS files
- Images

HTML is not cached to ensure updates are seen immediately.

### CDN (Optional)

For better global performance, consider using a CDN:
- CloudFront (AWS)
- Cloudflare
- Fastly
- Built-in with Netlify/Vercel

---

## Security Checklist

- [ ] Use HTTPS (SSL/TLS certificate)
- [ ] Set security headers (X-Frame-Options, etc.)
- [ ] Enable CORS only for necessary origins
- [ ] Keep server software updated
- [ ] Use a firewall (ufw, firewalld)
- [ ] Regular backups
- [ ] Monitor server logs

---

## Updating the Deployment

```bash
# 1. Pull latest changes
git pull

# 2. Rebuild
./deploy.sh

# 3. Upload to server
# Nginx/Apache:
sudo cp -r dist/* /var/www/tei-viewer/dist/

# Cloud platforms:
# Re-run deployment command (netlify deploy, vercel, etc.)

# 4. Clear CDN cache if using one
```

---

## Support

For issues specific to:
- **TEI Viewer application:** Check the main README.md
- **Server configuration:** Consult your server documentation
- **Cloud platforms:** Check their respective documentation

---

## Example Production Setup

A typical production setup might look like:

```
┌─────────────────┐
│   Domain Name   │
│ tei.example.com │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Cloudflare    │ ◄── CDN + DDoS protection
│   (DNS + CDN)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Nginx Server   │ ◄── SSL termination, serving static files
│  Ubuntu 22.04   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  /var/www/      │ ◄── TEI Viewer dist/ files
│  tei-viewer/    │
└─────────────────┘
```

Good luck with your deployment!