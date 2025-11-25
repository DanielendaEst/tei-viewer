# Quick Testing Guide for Responsive Design

## Branch
`mobile-responsive-design`

## Quick Start

```bash
cd tei-viewer
trunk serve --port 8080 --open
```

Then open DevTools (F12) and toggle Device Toolbar (Ctrl+Shift+M or Cmd+Shift+M on Mac).

## Test Checklist

### 🖥️ Desktop (1920x1080)
- [ ] Image and text panels side-by-side
- [ ] All controls visible in one row
- [ ] No scrolling needed for controls
- [ ] Legend panel displays nicely
- [ ] Font size comfortable (18px base)

### 📱 Tablet (1024x768)
- [ ] Image panel stacks above text panels
- [ ] Image height ~40vh (300-400px)
- [ ] Controls still horizontal but may wrap
- [ ] Selectors may stack
- [ ] No overlapping elements

### 📱 iPad Portrait (768x1024)
- [ ] Everything stacks vertically
- [ ] Image panel ~30vh (250-300px)
- [ ] Controls in centered row(s)
- [ ] Buttons easy to tap (44px min)
- [ ] Text readable (16px base)
- [ ] Legend single column

### 📱 iPhone (375x667)
- [ ] Compact vertical layout
- [ ] Image ~30vh (200px min)
- [ ] All buttons touchable
- [ ] No horizontal scroll
- [ ] Text content scrolls smoothly
- [ ] Controls wrap properly

### 📱 Small Phone (320x568)
- [ ] Ultra-compact layout works
- [ ] Image ~25vh (180px min)
- [ ] All content accessible
- [ ] Font size 14px still readable
- [ ] No UI breaking/overlapping

## Key Areas to Test

### 1. Header & Footer
- Text doesn't overflow
- Padding adjusts for screen
- Subtitle visible on mobile

### 2. Project/Page Selectors
- Dropdowns full-width on small screens
- Labels stack above selects
- Touch-friendly hit areas

### 3. Image Panel
- Maintains aspect ratio
- Zoom controls work
- Pan/drag works with touch
- No image overflow

### 4. Text Panels
- Scroll smoothly with momentum
- Line numbers don't overlap text
- Semantic markup visible/clickable
- Footnotes accessible

### 5. Controls Panel
- Buttons wrap cleanly
- View toggles centered on mobile
- Zoom controls accessible
- All buttons ≥44px height

### 6. Legend Panel
- Doesn't overflow screen
- Close button works
- Single column on mobile
- Swatches visible

### 7. Page Navigator
- Buttons don't crowd
- Page numbers readable
- Previous/Next work
- Fits in viewport

## Common Issues to Watch For

❌ **Horizontal scrolling** - Should never happen
❌ **Overlapping buttons** - Check controls panel
❌ **Tiny text** - Should scale appropriately
❌ **Cramped content** - Padding should adjust
❌ **Image too large** - Should fit in allocated height
❌ **Unclickable elements** - Touch targets too small
❌ **Legend overflow** - Should fit or scroll within panel

## Browser Testing

### Desktop Browsers
- Chrome (DevTools responsive mode)
- Firefox (Responsive Design Mode)
- Safari (Web Inspector)
- Edge (DevTools)

### Mobile Browsers (Real Devices)
- Safari iOS
- Chrome Android
- Firefox Mobile
- Samsung Internet

## Performance Checks

- [ ] Smooth scrolling on mobile
- [ ] No jank when resizing
- [ ] Images load properly
- [ ] No layout shift during load
- [ ] Touch interactions responsive

## Orientation Test

Test both orientations on tablets/phones:
- Portrait (default)
- Landscape (may need different rules)

## Accessibility

- [ ] All interactive elements keyboard accessible
- [ ] Focus visible on buttons
- [ ] Text contrast sufficient
- [ ] Touch targets ≥44px
- [ ] Semantic HTML maintained

## Screenshot Locations for Testing

1. Home page with project list
2. Viewer with image + single text panel
3. Viewer with image + dual text panels
4. Controls panel (mobile view)
5. Legend panel open
6. Page navigator
7. Metadata popup (if applicable)

## Known Limitations

- Very old browsers (<2020) may not support all CSS features
- Some CSS Grid features require modern browser
- `-webkit-overflow-scrolling` is iOS-specific

## Quick Fixes if Issues Found

### Overlapping elements
→ Check padding/margin in media query for that breakpoint

### Text too small
→ Adjust `font-size` in body or specific element media query

### Buttons not touch-friendly
→ Ensure `min-height: 44px` and sufficient padding

### Horizontal scroll
→ Check for `overflow-x: hidden` on html/body
→ Look for fixed-width elements not adapting

### Image overflow
→ Verify min-height/height constraints in image-panel media queries

## Success Criteria

✅ Zero overlapping UI elements at any screen size
✅ No horizontal scrolling on any device
✅ All buttons easily tappable on touch devices
✅ Text readable without zooming
✅ Smooth scrolling experience
✅ Layout adapts gracefully through all breakpoints
✅ Content remains accessible and functional

## Report Issues

If you find problems:
1. Note the screen size/breakpoint
2. Take a screenshot
3. Describe expected vs actual behavior
4. Note browser/device used

Example:
```
Issue: Buttons overlap at 650px width
Browser: Chrome 120
Expected: Buttons should wrap
Actual: Buttons crowd and overlap
Screenshot: attached
```
