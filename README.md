# TEI-XML Viewer

A modern web-based viewer for TEI-XML documents with image-text synchronization, built with Rust, Yew, and WebAssembly.

## Features

- 📖 **Dual View**: Display diplomatic editions and translations side-by-side
- 🖼️ **Image Synchronization**: Interactive highlighting between text and manuscript images
- 🔍 **Zoom & Pan**: Smooth image navigation with mouse wheel and drag
- 💬 **Commentary System**: Rich HTML commentary popups with auto-open and fallback support
- 🎨 **Semantic Markup**: Visual rendering of TEI elements (abbreviations, corrections, names, etc.)
- 📱 **Responsive**: Works on desktop and mobile devices with mobile pan & zoom
- 🚀 **Fast**: WebAssembly-powered performance
- 📚 **Multi-Project**: Support for multiple manuscript projects with dynamic loading

## Interface

The viewer provides an intuitive interface with:

- **View Controls**: Toggle between diplomatic edition, translation, or both views
- **Commentary Button**: Access rich HTML commentary for scholarly analysis
- **Image Controls**: Zoom in/out, metadata, and color legend toggles
- **Project Selector**: Switch between manuscript projects
- **Page Navigation**: Browse through manuscript pages
- **Interactive Highlighting**: Click or hover on text to highlight corresponding image zones

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Trunk](https://trunkrs.dev/): `cargo install trunk`
- [wasm32-unknown-unknown target](https://rustwasm.github.io/docs/book/game-of-life/setup.html): `rustup target add wasm32-unknown-unknown`

### Development

```bash
# Clone the repository
git clone <your-repo-url>
cd tei-viewer

# IMPORTANT: Create your projects folder first!
# The projects/ folder is gitignored and contains your manuscript data.
# You need to create it with your TEI files, images, and manifests.
# See "Adding Projects" section below for structure.

# Example: Create a test project
mkdir -p projects/MyProject/images
# ... add manifest.json, XML files, and images ...

# Sync project files (copies from projects/ to public/projects/)
./sync_projects.sh

# Start development server
./start.sh
# Or manually:
trunk serve
```

**⚠️ Important**: The `projects/` folder is **not tracked in git**. You must create it locally with your manuscript data before running the viewer. See the "Adding Projects" section below for the required structure.

The viewer will be available at `http://127.0.0.1:8080`

### Production Build

```bash
# Sync projects and build
./sync_projects.sh
trunk build

# Or use the deploy script
./deploy.sh
```

The production files will be in the `dist/` directory.

## Project Structure

```
tei-viewer/
├── src/
│   ├── main.rs                    # Application entry point
│   ├── components/
│   │   └── tei_viewer.rs          # Main viewer component
│   ├── tei_parser.rs              # TEI-XML parser
│   ├── tei_data.rs                # Data structures
│   └── project_config.rs          # Project configuration types
├── projects/                      # SOURCE OF TRUTH for project data
│   ├── PGM-XIII/
│   │   ├── manifest.json          # Project metadata
│   │   ├── commentary.html        # Commentary content (optional)
│   │   ├── p1_dip.xml             # Diplomatic edition, page 1
│   │   ├── p1_trad.xml            # Translation, page 1
│   │   └── images/
│   │       ├── p1.jpg             # Image for page 1
│   │       └── ...
│   └── [OtherProjects]/
├── public/                        # Generated (do not edit directly)
│   └── projects/                  # Auto-synced from projects/
├── static/
│   └── styles.css                 # Application styles
├── index.html                     # HTML template
├── sync_projects.sh               # Project sync script (REQUIRED) - copies XML, images, manifests, and commentary.html
├── start.sh                       # Development startup script
├── deploy.sh                      # Production build script
└── deploy-gh-pages.sh             # GitHub Pages deployment script
```

## Adding Projects

### ⚠️ First Time Setup

The `projects/` folder is **gitignored** and not included in the repository. You must create it locally:

```bash
cd tei-viewer
mkdir -p projects
```

Then add your project folders inside following the structure below.

### 1. Create Project Structure

Create a new folder in `projects/` with this structure:

```
projects/
└── YourProject/
    ├── manifest.json              # Required
    ├── commentary.html            # Commentary content (optional)
    ├── p1_dip.xml                 # Diplomatic edition
    ├── p1_trad.xml                # Translation (optional)
    ├── p2_dip.xml                 # Additional pages...
    └── images/
        ├── p1.jpg                 # Image for page 1
        └── p2.jpg                 # Image for page 2
```

### 2. Create manifest.json

Each project **must** have a `manifest.json` file:

```json
{
  "id": "YourProject",
  "name": "Your Project Name",
  "description": "Brief description of your project",
  "metadata": {
    "author": "Author Name",
    "editor": "Editor Name",
    "collection": "Collection Name",
    "institution": "Institution Name",
    "repository": "https://repository-url.com",
    "country": "Country",
    "settlement": "City",
    "language": "Language (e.g., Ancient Greek)",
    "date_range": "Date range",
    "siglum": "Manuscript siglum",
    "old_catalog": "Old catalog number",
    "museum_id": "Museum ID",
    "origin_place": "Place of origin"
  },
  "pages": [
    {
      "number": 1,
      "label": "Folio 1r",
      "has_diplomatic": true,
      "has_translation": true,
      "has_image": true,
      "notes": "Optional notes"
    },
    {
      "number": 2,
      "label": "Folio 1v",
      "has_diplomatic": true,
      "has_translation": false,
      "has_image": true
    }
  ],
  "version": "1.0.0",
  "last_updated": "2024-01-01"
}
```

### 3. File Naming Conventions

**IMPORTANT**: Files must follow these exact naming patterns:

- **Commentary**: `commentary.html` (optional)
  - Rich HTML content with academic commentary
  - Auto-opens on first app load
  - Falls back to "Sin comentario" if missing

- **XML files**: `p{number}_{type}.xml`
  - `p1_dip.xml` = Diplomatic edition, page 1
  - `p1_trad.xml` = Translation, page 1
  - `p2_dip.xml` = Diplomatic edition, page 2
  
- **Images**: `p{number}.jpg`
  - `p1.jpg` = Image for page 1
  - `p2.jpg` = Image for page 2

### 4. Register Project

Edit `src/main.rs` and add your project ID to the list:

```rust
async fn load_all_manifests() -> Result<Vec<ProjectConfig>, String> {
    let project_ids = vec![
        "PGM-XIII", 
        "Tractatus-Fascinatione", 
        "Chanca",
        "YourProject"  // Add your project here
    ];
    // ...
}
```

### 5. Sync and Build

```bash
# Sync projects to public folder (includes commentary.html files)
./sync_projects.sh

# Rebuild
trunk build

# Or start dev server
trunk serve
```

## Deployment

### GitHub Pages Deployment

GitHub Pages requires special configuration because it serves files from a subdirectory.

#### How GitHub Pages Serves Your Files

**Everything is managed automatically!** When you deploy to GitHub Pages:

1. **The deploy script syncs your projects**:
   ```bash
   ./sync_projects.sh  # Copies projects/ → public/projects/
   ```

2. **Trunk bundles everything into `dist/`**:
   ```
   dist/
   ├── index.html, *.wasm, *.js  # Your app
   └── public/projects/          # ← Your static files!
       ├── PGM-XIII/
       │   ├── manifest.json
       │   ├── p1_dip.xml
       │   ├── p1_trad.xml
       │   └── images/p1.jpg     # All images included
       ├── Chanca/
       └── Tractatus-Fascinatione/
   ```

3. **GitHub Pages serves `dist/` as a static website**:
   - Your app: `https://username.github.io/tei-viewer/`
   - XML files: `https://username.github.io/tei-viewer/public/projects/PGM-XIII/p1_dip.xml`
   - Images: `https://username.github.io/tei-viewer/public/projects/PGM-XIII/images/p1.jpg`

**You don't need a separate server** - GitHub Pages hosts your XMLs, images, and WASM app together!

**⚠️ Important**: Before deploying, make sure you have created the `projects/` folder locally with your manuscript data. The deployment script will sync and bundle it automatically.

#### Option 1: Using deploy-gh-pages.sh (Recommended)

```bash
# Build and deploy to gh-pages branch
./deploy-gh-pages.sh
```

This script automatically:
1. Syncs projects from `projects/` to `public/projects/` (includes commentary.html)
2. Builds the application with `--public-url /tei-viewer/`
3. Adds `.nojekyll` file (prevents Jekyll processing)
4. Commits to `gh-pages` branch
5. Pushes to GitHub

#### Option 2: Manual GitHub Pages Setup

1. **Sync projects**:
   ```bash
   ./sync_projects.sh
   ```

2. **Build with correct base path**:
   ```bash
   trunk build --release --public-url /your-repo-name/
   ```

3. **Add .nojekyll to dist/**:
   ```bash
   touch dist/.nojekyll
   ```

4. **Deploy dist/ folder**:
   ```bash
   cd dist
   git init
   git add -A
   git commit -m 'Deploy to GitHub Pages'
   git push -f git@github.com:username/repo.git main:gh-pages
   ```

5. **Configure GitHub repository**:
   - Go to Settings → Pages
   - Source: Deploy from branch
   - Branch: `gh-pages` / `root`
   - Save

#### Important Notes for GitHub Pages

- **Public URL**: Must match your repository name (e.g., `/tei-viewer/`)
- **`.nojekyll` file**: Required to prevent Jekyll from processing files
- **Branch**: Deploy from `gh-pages` branch, not `main`
- **All files bundled**: XMLs, images, and app are deployed together (no CORS issues!)
- **Project data**: Must be in `projects/` folder before running deploy script

#### Adding/Updating Projects on GitHub Pages

```bash
# 1. Add/edit files in projects/YourProject/
# 2. Deploy (sync happens automatically)
./deploy-gh-pages.sh
```

The script handles everything - you don't need to manually sync or manage static files!

### Standard Web Server Deployment

For Apache, Nginx, or other web servers:

```bash
# Build for production
./deploy.sh

# Copy dist/ contents to your web server
rsync -avz dist/ user@server:/var/www/html/
```

**Server Configuration Requirements**:

1. **MIME Types**: Ensure `.wasm` files are served with correct MIME type:
   ```
   application/wasm
   ```

2. **Headers** (optional but recommended):
   ```
   Cross-Origin-Opener-Policy: same-origin
   Cross-Origin-Embedder-Policy: require-corp
   ```

3. **Compression**: Enable gzip/brotli for `.wasm` and `.js` files

#### Nginx Example

```nginx
server {
    listen 80;
    server_name your-domain.com;
    root /var/www/tei-viewer;
    index index.html;

    # WASM MIME type
    types {
        application/wasm wasm;
    }

    # Compression
    gzip on;
    gzip_types application/wasm application/javascript text/css;

    # SPA fallback
    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

#### Apache Example

```apache
<VirtualHost *:80>
    ServerName your-domain.com
    DocumentRoot /var/www/tei-viewer

    # WASM MIME type
    AddType application/wasm .wasm

    # Compression
    <IfModule mod_deflate.c>
        AddOutputFilterByType DEFLATE application/wasm
        AddOutputFilterByType DEFLATE application/javascript
        AddOutputFilterByType DEFLATE text/css
    </IfModule>

    # SPA fallback
    <Directory /var/www/tei-viewer>
        RewriteEngine On
        RewriteBase /
        RewriteCond %{REQUEST_FILENAME} !-f
        RewriteCond %{REQUEST_FILENAME} !-d
        RewriteRule . /index.html [L]
    </Directory>
</VirtualHost>
```

### Docker Deployment

```bash
# Build Docker image
docker build -t tei-viewer .

# Run container
docker run -p 8080:80 tei-viewer
```

The included `Dockerfile` uses Nginx to serve the static files.

## Workflow

### Day-to-day Development

1. **Edit project files** in `projects/YourProject/`
2. **Sync changes**: `./sync_projects.sh`
3. **View changes**: `trunk serve` (auto-reloads)

### Adding a New Page

1. Add XML files: `projects/YourProject/p3_dip.xml`, `p3_trad.xml`
2. Add image: `projects/YourProject/images/p3.jpg`
3. Update `manifest.json`:
   ```json
   {
     "number": 3,
     "label": "Folio 2r",
     "has_diplomatic": true,
     "has_translation": true,
     "has_image": true
   }
   ```
4. Run: `./sync_projects.sh`
5. Rebuild: `trunk build`

## TEI-XML Support

### Supported Elements

- `<abbr>` / `<expan>` - Abbreviations and expansions
- `<sic>` / `<corr>` - Original errors and corrections
- `<orig>` / `<reg>` - Original and regularized forms
- `<unclear>` - Unclear text
- `<persName>` - Person names
- `<placeName>` - Place names
- `<num>` - Numbers
- `<ref>` - References
- `<hi rend="...">` - Highlighted text (bold, italic, underline, superscript, subscript)
- `<note>` - Footnotes and annotations
- `<lb>` - Line breaks
- `<zone>` - Facsimile zones for highlighting

### Commentary System

The viewer supports rich HTML commentary for each project:

- **File location**: `projects/ProjectName/commentary.html`
- **Format**: Full HTML document with styling
- **Auto-display**: Opens automatically on first app load
- **Toggle button**: "Comentario" button in view controls
- **Fallback**: Shows "Sin comentario" if file doesn't exist
- **Responsive**: 85% screen coverage with dark theme styling

**Example commentary.html**:
```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Commentary</title>
</head>
<body>
    <h1>Academic Commentary</h1>
    <p>Your scholarly analysis here...</p>
    <blockquote>Citations and references...</blockquote>
</body>
</html>
```

### Facsimile Linking

Images are linked to text using TEI `<facsimile>` and `<zone>` elements:

```xml
<facsimile>
  <graphic url="images/p1.jpg" width="2479px" height="3508px"/>
  <surface>
    <zone xml:id="z_line_1" points="100,100 500,100 500,150 100,150"/>
    <!-- More zones... -->
  </surface>
</facsimile>

<text>
  <body>
    <lb n="1" facs="#z_line_1"/>Line of text here...
  </body>
</text>
```

When hovering over text, the corresponding zone highlights on the image.

## Troubleshooting

### Commentary Not Showing

- Check file exists: `projects/YourProject/commentary.html`
- Verify HTML is valid (no syntax errors)
- Run `./sync_projects.sh` to copy to public folder
- Check browser console for loading errors
- If no commentary exists, "Sin comentario" should display

### Images Don't Display

- Check file naming: Must be `p{number}.jpg`
- Check location: Must be in `projects/YourProject/images/`
- Run `./sync_projects.sh` to copy to public folder
- Check browser console for 404 errors

### Highlights Misaligned

The viewer automatically scales zone coordinates from TEI-declared dimensions to actual image dimensions. If highlights are still off:

1. Verify `<graphic>` element has correct `width` and `height` with units (e.g., `"2479px"`)
2. Verify `<zone>` coordinates match the declared dimensions
3. Check browser console for coordinate scaling logs

**Common Issue**: If your image was resized but the XML coordinates weren't updated:

The XML might declare `<graphic width="2479px" height="3508px"/>` but the actual image is smaller (e.g., `960×1358`). This causes misaligned highlights because the zone coordinates are in the original large image space.

**Solution**: You'll need to rescale the coordinates manually or write a script to:
- Update `<graphic>` dimensions to match actual image size
- Scale all zone coordinates proportionally (new = old × scale_factor)
- Update image filename references if needed

### Project Not Appearing

1. Check `manifest.json` is valid JSON
2. Verify project ID is added to `src/main.rs` in `load_all_manifests()`
3. Run `./sync_projects.sh`
4. Rebuild: `trunk build`
5. Clear browser cache

### Build Errors

**wasm-opt error during release build**:
```bash
# Use dev build instead
trunk build
```

**Missing dependencies**:
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

### Commentary Features Not Working

**Commentary button not visible**:
- Check that you're on the latest version with commentary feature
- Commentary button is always visible regardless of file existence

**Commentary popup not opening**:
- Click "Comentario" button in view toggles
- Check browser console for JavaScript errors
- Verify HTML content is valid

**Wrong styling in commentary popup**:
- Commentary uses dark theme colors matching the app
- Custom CSS in commentary.html may conflict with app styles
- Use relative units (em, rem) rather than fixed pixels

### GitHub Pages Not Working

- Verify `--public-url` matches your repo name
- Check `.nojekyll` file exists in deployed folder
- Ensure `gh-pages` branch is configured in repository settings
- Wait a few minutes for GitHub to rebuild

## Current Projects

- **PGM-XIII**: Papyri Graecae Magicae XIII (Ancient Greek magical papyrus)
- **Tractatus-Fascinatione**: Tractatus de Fascinatione
- **Chanca**: Chanca manuscript

## Architecture

### Technology Stack

- **Frontend Framework**: [Yew](https://yew.rs/) - Rust/WebAssembly framework
- **Build Tool**: [Trunk](https://trunkrs.dev/) - WASM web application bundler
- **Language**: Rust (compiled to WebAssembly)
- **XML Parsing**: quick-xml
- **HTTP**: gloo-net

### Key Components

1. **App** (`src/main.rs`): Main application, manages project/page selection
2. **TeiViewer** (`src/components/tei_viewer.rs`): Displays TEI documents with image sync
3. **TEI Parser** (`src/tei_parser.rs`): Parses TEI-XML into data structures
4. **Project Config** (`src/project_config.rs`): Handles manifest.json loading

### State Management

- Projects loaded dynamically at startup from manifest.json files
- Page changes trigger XML/image reloads
- Image dimensions auto-detected for coordinate scaling
- Hover/click state managed for highlight synchronization

## Contributing

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` to check for warnings
- Follow Rust naming conventions

### Adding Features

1. Create a feature branch
2. Make changes and test locally
3. Update this README if needed
4. Submit a pull request

## License

[Your License Here]

## Known Issues & Solutions

### Coordinate Scaling for Resized Images

If your TEI files were generated from PAGE-XML for a high-resolution image, but you're using a smaller version of the image in the viewer, you'll need to rescale the coordinates.

**Symptoms**:
- Highlights appear in wrong locations
- Console shows: "Dimensiones Declaradas: 2479 × 3508" but "Dimensiones Intrínsecas: 960 × 1358"
- Zone coordinates exceed image dimensions

**Fix**: Use `fix_tractatus_coords.py` as a template to create a rescaling script for your project. The script should:
1. Calculate scale factors: `scale_x = new_width / old_width`
2. Update `<graphic width="..." height="...">` to actual image size
3. Scale all zone points: `new_point = old_point * scale_factor`
4. Update image filename if needed

This issue typically occurs when:
- Images are downsampled for web delivery
- PAGE-XML was created on original high-res scans
- TEI conversion didn't account for image resizing

## FAQ

### Do I need a separate server for my XML files and images?

**No!** GitHub Pages serves everything together. The `deploy-gh-pages.sh` script:
1. Syncs your `projects/` folder to `public/projects/`
2. Bundles everything (app + XMLs + images) into `dist/`
3. Deploys `dist/` to GitHub Pages

Your XMLs and images are served from the same domain as your app, so there are no CORS issues.

### Where should I keep my project files?

**The `projects/` folder is gitignored and must be created locally.** Keep all your project data in `tei-viewer/projects/`:

```
projects/
├── PGM-XIII/
│   ├── manifest.json
│   ├── commentary.html
│   ├── p1_dip.xml
│   ├── p1_trad.xml
│   └── images/p1.jpg
└── YourProject/
    └── ...
```

The `projects/` folder is **gitignored** - it's not tracked in the repository. When you deploy to GitHub Pages, the script automatically copies this to `public/projects/` and includes it in the deployment.

### Why isn't there a projects/ folder when I clone?

The `projects/` folder is **gitignored** because it contains your specific manuscript data. This keeps the repository clean and allows each user to have their own projects.

**You must create it yourself**:
```bash
mkdir -p projects/YourProject/images
# Add manifest.json, commentary.html, XML files, and images
```

See the "Adding Projects" section for the complete structure.

### How do I update projects on GitHub Pages?

Just run the deploy script:

```bash
# Edit files in projects/YourProject/
./deploy-gh-pages.sh
```

The script handles:
- ✅ Syncing projects
- ✅ Building the app
- ✅ Deploying to gh-pages branch
- ✅ No manual file management needed!

### What about the `public/projects/` folder?

This is **auto-generated** by `sync_projects.sh` and should not be edited directly:
- Source of truth: `projects/` (gitignored, managed locally)
- Generated copy: `public/projects/` (gitignored, created by sync)
- Deployed version: Bundled into `dist/` by Trunk, then deployed

Always edit files in `projects/` and run `./sync_projects.sh` or deployment scripts.

### Can I use a different static file host?

Yes, but you don't need to! GitHub Pages:
- Hosts unlimited static files (within repo size limits)
- Serves XMLs, images, and WASM together
- No additional configuration needed
- Free for public repositories

For private projects or custom domains, you can deploy `dist/` to any static host (Netlify, Vercel, S3, nginx, Apache).

## Credits

Developed for digital humanities manuscript transcription and display.

## Support

For issues or questions:
- Open an issue on GitHub
- Check the troubleshooting section above
- Review the browser console for errors

---

**Note**: The `projects/` folder is the source of truth. The `public/projects/` folder is auto-generated by `sync_projects.sh` and should not be edited directly.