# New Project Checklist

Quick reference guide for adding a new TEI project to the viewer.

## Prerequisites

- [ ] Python 3 installed (for fix scripts if needed)
- [ ] Rust and Trunk installed
- [ ] Git repository cloned

## Step 1: Prepare Your Files

### Required Files

Create in `tei-viewer/projects/YourProjectName/`:

- [ ] `manifest.json` - Project metadata and page information
- [ ] `p1_dip.xml` - Diplomatic edition for page 1
- [ ] `p1_trad.xml` - Translation for page 1 (optional)
- [ ] `images/p1.jpg` - Image for page 1

### File Naming Convention

**IMPORTANT**: Follow these exact patterns:

```
XML files:  p{number}_{type}.xml
Examples:   p1_dip.xml, p1_trad.xml, p2_dip.xml

Images:     p{number}.jpg
Examples:   p1.jpg, p2.jpg, p3.jpg
```

## Step 2: Create manifest.json

```json
{
  "id": "YourProjectName",
  "name": "Your Project Display Name",
  "description": "Brief description of the project",
  "metadata": {
    "author": "Author Name",
    "editor": "Editor Name",
    "collection": "Collection Name",
    "institution": "Institution Name",
    "repository": "https://repository-url.com",
    "country": "Country",
    "settlement": "City",
    "language": "Language (e.g., Ancient Greek (grc))",
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
      "notes": "Optional notes about this page"
    }
  ],
  "version": "1.0.0",
  "last_updated": "2024-01-01"
}
```

## Step 3: Verify TEI Files

### Check Image Dimensions

- [ ] Get actual image dimensions: `file images/p1.jpg`
- [ ] Verify `<graphic>` element in XML:
  ```xml
  <graphic url="images/p1.jpg" width="960px" height="1358px"/>
  ```
- [ ] **CRITICAL**: `width` and `height` MUST match actual image size

### Check Zone Coordinates

- [ ] Zone coordinates should be in the same space as image dimensions
- [ ] If coordinates exceed image dimensions → need to rescale (see Fix Scripts)

### Check Line Order

- [ ] Lines should appear top-to-bottom in the manuscript
- [ ] If highlights "jump" around → need to reorder (see Fix Scripts)

## Step 4: Register Project

Edit `src/main.rs` around line 211:

```rust
async fn load_all_manifests() -> Result<Vec<ProjectConfig>, String> {
    let project_ids = vec![
        "PGM-XIII", 
        "Tractatus-Fascinatione", 
        "Chanca",
        "YourProjectName"  // ← Add your project ID here
    ];
    // ...
}
```

## Step 5: Sync and Build

```bash
# Sync projects to public folder
./sync_projects.sh

# Build the application
trunk build

# Test locally
trunk serve
```

Open http://127.0.0.1:8080 and verify:
- [ ] Project appears in dropdown
- [ ] Pages load correctly
- [ ] Images display
- [ ] Highlights align with image text
- [ ] Highlights match correct text lines

## Common Issues & Fixes

### Issue: Highlights in Wrong Position

**Symptom**: Zone overlays don't align with text in image

**Cause**: Image dimensions in XML don't match actual image

**Fix**:
```bash
# Edit fix_tractatus_coords.py:
OLD_WIDTH = 2479   # From <graphic width="...">
OLD_HEIGHT = 3508
NEW_WIDTH = 960    # From: file images/p1.jpg
NEW_HEIGHT = 1358

# Run fix
python3 fix_tractatus_coords.py
./sync_projects.sh
trunk build
```

### Issue: Highlights Jump Between Lines

**Symptom**: Highlight positions correct but match wrong text

**Cause**: Lines in wrong reading order (not sorted by Y-coordinate)

**Fix**:
```bash
# Edit fix_tractatus_order.py to target your project
python3 fix_tractatus_order.py
./sync_projects.sh
trunk build
```

### Issue: Project Doesn't Appear

**Checklist**:
- [ ] Project ID added to `src/main.rs`
- [ ] `manifest.json` is valid JSON
- [ ] Ran `./sync_projects.sh`
- [ ] Rebuilt with `trunk build`
- [ ] Cleared browser cache

### Issue: Image Not Loading

**Checklist**:
- [ ] Image named `p{number}.jpg` (e.g., `p1.jpg` not `page1.jpg`)
- [ ] Image in `images/` subdirectory
- [ ] `<graphic url="images/p1.jpg">` in XML (not full path)
- [ ] Ran `./sync_projects.sh`

## Step 6: Test Everything

- [ ] Switch between projects - images change correctly
- [ ] Switch between pages - content updates
- [ ] Hover over text - highlights appear on image
- [ ] Highlights align with correct manuscript regions
- [ ] Highlights stay with correct text lines
- [ ] Metadata displays correctly
- [ ] Both diplomatic and translation load (if applicable)

## Step 7: Deploy

```bash
# For GitHub Pages
./deploy-gh-pages.sh

# For standard web server
./deploy.sh
# Then copy dist/ to your server
```

## Project Structure Summary

```
projects/YourProjectName/
├── manifest.json              ✓ Required
├── p1_dip.xml                 ✓ Required (diplomatic)
├── p1_trad.xml                ○ Optional (translation)
├── p2_dip.xml                 ○ Add more pages
├── p2_trad.xml
└── images/
    ├── p1.jpg                 ✓ Required
    └── p2.jpg                 ○ One per page
```

## Validation Checklist

Before committing:

- [ ] All file names follow convention
- [ ] `manifest.json` validates as JSON
- [ ] Image dimensions match XML `<graphic>` element
- [ ] Zone coordinates within image bounds
- [ ] Lines in correct reading order
- [ ] Project registered in `src/main.rs`
- [ ] `./sync_projects.sh` executed
- [ ] Build succeeds: `trunk build`
- [ ] Tested in browser
- [ ] No console errors

## Getting Help

If you encounter issues:

1. Check browser console for errors
2. Review TRACTATUS_FIX.txt for similar issues
3. Verify file naming matches examples
4. Ensure all required files are present
5. Run `./sync_projects.sh` again
6. Clear browser cache and reload

## Tips

- **Start simple**: Add one page first, then expand
- **Test early**: Build and test after each page
- **Use templates**: Copy structure from existing projects (PGM-XIII, Chanca)
- **Document issues**: Note any fixes needed for future reference
- **Version control**: Commit working states frequently

---

**Last Updated**: 2024-11-25