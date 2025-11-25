# TEI Viewer - Deployment Decision Tree

## 🤔 Which Deployment Method Should I Use?

```
START: Where do you want to deploy?
│
├─ 📱 Just want to test/demo quickly? (2-5 minutes)
│  │
│  ├─ Local testing only
│  │  └─ ✅ USE: Python Server
│  │     └─ `./deploy.sh && python3 -m http.server -d dist 8000`
│  │
│  └─ Share with others / public demo
│     └─ ✅ USE: Netlify Drop
│        └─ `./deploy.sh` then drag dist/ to app.netlify.com/drop
│
├─ 🐳 Want the easiest server setup? (5-10 minutes)
│  └─ ✅ USE: Docker
│     ├─ Local: `docker-compose up -d`
│     ├─ VPS: Install Docker, copy files, `docker-compose up -d`
│     └─ Pros: Consistent, easy, portable
│
├─ 💰 Have a VPS/dedicated server? (15-30 minutes)
│  │
│  ├─ Familiar with Nginx?
│  │  └─ ✅ USE: Nginx
│  │     ├─ Best performance
│  │     ├─ Easy SSL with Let's Encrypt
│  │     └─ See: deployment/nginx.conf
│  │
│  └─ Familiar with Apache?
│     └─ ✅ USE: Apache
│        ├─ Works on shared hosting
│        ├─ .htaccess support
│        └─ See: deployment/apache.conf
│
├─ ☁️ Want automatic scaling/CDN? (2-10 minutes)
│  │
│  ├─ Free tier OK?
│  │  ├─ ✅ Netlify (recommended for simplicity)
│  │  ├─ ✅ Vercel (recommended for speed)
│  │  └─ ✅ GitHub Pages (if already on GitHub)
│  │
│  └─ Need enterprise features?
│     ├─ ✅ AWS S3 + CloudFront
│     ├─ ✅ Google Cloud Storage + CDN
│     └─ ✅ Azure Static Web Apps
│
└─ 🏢 Internal/institutional deployment?
   ├─ Have IT support?
   │  └─ ✅ USE: Institutional servers (Nginx/Apache)
   │     └─ Provide them with deployment/README.md
   │
   └─ Self-managed?
      └─ ✅ USE: Docker on institutional VPS
         └─ Easiest to maintain
```

---

## 📊 Comparison Matrix

| Criteria | Docker | Nginx | Apache | Netlify | Python |
|----------|--------|-------|--------|---------|--------|
| **Setup Time** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Performance** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| **Scalability** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐ |
| **Cost** | Free | $5-10/mo | $5-10/mo | Free tier | Free |
| **SSL Setup** | Manual | Easy | Easy | Automatic | N/A |
| **Production Ready** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No |
| **Maintenance** | Low | Medium | Medium | None | N/A |
| **Best For** | Dev/Prod | VPS Prod | Shared Host | Public Sites | Testing |

---

## 🎯 Recommended Paths

### Path 1: Quick Demo (Fastest)
```bash
# 2 minutes
./deploy.sh
# Drag dist/ to app.netlify.com/drop
# Done! You have a public URL
```

### Path 2: Docker Development/Staging (Easiest)
```bash
# 5 minutes
docker-compose up -d
# Access at localhost:8080
# Deploy same way on any server
```

### Path 3: Professional Production (Best Performance)
```bash
# 30 minutes

# 1. Get a VPS ($5/month - DigitalOcean, Linode, etc.)
# 2. Install Nginx
# 3. Build and upload
./deploy.sh
scp -r dist user@server:/var/www/tei-viewer/

# 4. Configure Nginx (use provided config)
sudo cp deployment/nginx.conf /etc/nginx/sites-available/tei-viewer
sudo ln -s /etc/nginx/sites-available/tei-viewer /etc/nginx/sites-enabled/
sudo systemctl restart nginx

# 5. Add free SSL
sudo certbot --nginx -d your-domain.com

# Done! Production-ready deployment
```

### Path 4: Institutional/Academic (Most Common)
```bash
# Option A: Provide to IT department
# Give them: deployment/README.md + dist/ folder
# They deploy on institutional servers

# Option B: Docker on institutional server
docker-compose up -d
# Runs on any server with Docker
```

---

## 💡 Special Cases

### Case: "I have shared hosting"
➡️ **Use:** Apache  
➡️ **Why:** Most shared hosts run Apache  
➡️ **How:** Upload `dist/` via FTP, use `.htaccess`

### Case: "I need it free and public"
➡️ **Use:** Netlify or GitHub Pages  
➡️ **Why:** Free tier, automatic HTTPS, CDN  
➡️ **How:** `./deploy.sh` then one-click deploy

### Case: "I need maximum performance"
➡️ **Use:** Nginx + CDN (CloudFront/Cloudflare)  
➡️ **Why:** Fastest static file serving  
➡️ **How:** Nginx config + add CloudFlare DNS

### Case: "I need it to work everywhere"
➡️ **Use:** Docker  
➡️ **Why:** Same container runs anywhere  
➡️ **How:** `docker-compose up -d`

### Case: "I'm just testing locally"
➡️ **Use:** Python server or `trunk serve`  
➡️ **Why:** Simple, no configuration  
➡️ **How:** `python3 -m http.server -d dist 8000`

### Case: "Academic repository/archive"
➡️ **Use:** Institutional servers (Nginx/Apache) or Netlify  
➡️ **Why:** Long-term stability, institutional support  
➡️ **How:** Coordinate with IT or use academic Netlify plan

---

## 🚦 Quick Decision Flowchart

```
Do you have a server? 
├─ YES → Is Docker available?
│         ├─ YES → USE DOCKER ✅
│         └─ NO  → USE NGINX/APACHE ✅
│
└─ NO  → Is this for production?
          ├─ YES → USE NETLIFY/VERCEL ✅
          └─ NO  → USE PYTHON SERVER ✅
```

---

## 📞 Getting Help

Still not sure? Answer these questions:

1. **Budget?** Free / $5-10/month / Enterprise
2. **Technical skill?** Beginner / Intermediate / Expert
3. **Purpose?** Testing / Demo / Production / Archive
4. **Infrastructure?** None / Shared hosting / VPS / Cloud
5. **Audience?** Just me / Team / Public

### Based on answers:

- **Free + Beginner + Demo + None + Public** → Netlify Drop
- **$5/mo + Intermediate + Production + VPS + Public** → Nginx + Let's Encrypt
- **Free + Any + Testing + Any + Just me** → Python Server
- **Any + Beginner + Production + Any + Any** → Docker
- **Free + Any + Archive + Institution + Academic** → Institutional servers

---

## 📚 Next Steps

Once you've chosen:

1. Read the relevant section in `deployment/README.md`
2. Check the example configs in `deployment/`
3. Run `./deploy.sh` to build
4. Follow the deployment steps for your chosen method
5. Test the deployment
6. Set up monitoring/backups (for production)

---

**Remember:** All methods are valid! Choose based on:
- Your comfort level
- Available resources
- Production requirements
- Time constraints

**When in doubt, use Docker** - it's the easiest to set up and works everywhere.