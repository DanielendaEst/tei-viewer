# Mobile Image Pan & Zoom Implementation

## Branch: mobile-image-zoom

This implementation adds touch-based pan and pinch-zoom functionality to the TEI Viewer's image panel, enabling intuitive mobile interaction with manuscript images.

## Features Implemented

### 1. Pointer Event System
- **Multi-touch tracking**: Tracks up to multiple simultaneous touch points using pointer events
- **Pointer capture**: Ensures touch events continue to be received even when fingers move outside the container
- **Cross-platform**: Works with both touch devices and mouse input

### 2. Pan Functionality
- **Single-finger drag**: Pan the image by dragging with one finger
- **Smooth tracking**: Real-time coordinate updates for responsive movement
- **Momentum preservation**: Maintains image position when transitioning between gesture types

### 3. Pinch Zoom
- **Two-finger zoom**: Scale image using pinch gestures
- **Zoom center**: Zoom occurs around the midpoint between two fingers
- **Scale limits**: Constrained between 0.1x and 8.0x magnification
- **Smooth scaling**: Progressive zoom based on distance change between pointers

### 4. Touch Optimizations
- **Gesture prevention**: `touch-action: none` prevents browser default gestures
- **Event handling**: All touch events properly prevented from bubbling
- **State management**: Robust tracking of pointer states and transitions

## Technical Implementation

### Code Changes

#### 1. Message Types (`TeiViewerMsg`)
```rust
PointerDown(i32, i32, i32),  // pointer_id, x, y
PointerMove(i32, i32, i32),  // pointer_id, x, y
PointerUp(i32, i32, i32),    // pointer_id, x, y
PointerLeave(i32, i32, i32), // pointer_id, x, y
```

#### 2. State Management (`TeiViewer` struct)
```rust
pointers: Vec<(i32, (i32, i32))>,    // [(pointer_id, (x, y))]
last_pointer_distance: f64,          // For pinch zoom tracking
last_mouse_x: i32,                   // Last pointer position
last_mouse_y: i32,                   // Last pointer position
```

#### 3. Event Handlers
- **PointerDown**: Add pointer to tracking, initialize drag/zoom state
- **PointerMove**: Update pointer position, calculate pan/zoom deltas
- **PointerUp/Leave**: Remove pointer from tracking, reset states

#### 4. Zoom Center Calculation
```rust
// Zoom around gesture midpoint
let center_x = (p1.0 + p2.0) as f32 / 2.0;
let center_y = (p1.1 + p2.1) as f32 / 2.0;

// Adjust offset to maintain zoom center
let scale_change = new_scale / old_scale;
self.image_offset_x = center_x + (self.image_offset_x - center_x) * scale_change;
self.image_offset_y = center_y + (self.image_offset_y - center_y) * scale_change;
```

### Pointer Capture Implementation
```rust
// On pointer down - capture pointer
element.set_pointer_capture(e.pointer_id());

// On pointer up - release capture
element.release_pointer_capture(e.pointer_id());
```

## CSS Integration

Added to image container:
```css
touch-action: none;  /* Disable browser pan/zoom gestures */
```

## Compatibility

### Supported Browsers
- **Mobile Safari**: iOS 13+
- **Chrome Mobile**: Android 5.0+
- **Firefox Mobile**: Android 68+
- **Desktop**: All modern browsers with mouse fallback

### Input Methods
- **Touch**: Primary target - finger gestures
- **Mouse**: Maintained compatibility with existing mouse events
- **Stylus**: Supported on compatible devices

## Testing Recommendations

### Manual Testing
1. **Single-finger pan**:
   - Touch and drag image around viewport
   - Verify smooth movement in all directions

2. **Pinch zoom**:
   - Use two fingers to zoom in/out
   - Verify zoom occurs around gesture center
   - Test zoom limits (0.1x to 8.0x)

3. **Gesture transitions**:
   - Start with one finger, add second finger for zoom
   - Remove one finger while zooming, continue panning
   - Verify no jumps or unexpected behavior

4. **Edge cases**:
   - Rapid finger movements
   - Fingers leaving/re-entering container
   - Device rotation during gesture

### Browser Testing
- Test on actual mobile devices, not just desktop simulators
- Verify `touch-action: none` prevents browser zoom conflicts
- Test with different viewport sizes and orientations

## Performance Considerations

- **Event throttling**: Consider throttling pointer move events for very slow devices
- **Transform optimization**: Uses CSS transforms for hardware acceleration
- **Memory management**: Proper cleanup of pointer tracking arrays

## Future Enhancements

1. **Bounds checking**: Prevent image from being panned completely out of view
2. **Double-tap zoom**: Add quick zoom to fit/actual size
3. **Inertia**: Add momentum/inertia to pan gestures
4. **Accessibility**: Improve keyboard navigation support
5. **Gesture customization**: Allow configuration of zoom limits and sensitivity

## Development Notes

- **Coordinate system**: Uses client coordinates (screen pixels)
- **Transform origin**: Set to top-left (0, 0) for predictable scaling
- **Z-index**: Ensure image overlays remain properly layered during zoom
- **State consistency**: Proper reset of tracking state during pointer transitions

## Build & Deploy

```bash
# Build the application
trunk build

# Run development server
trunk serve --port 8080

# Test on mobile
# Use browser dev tools responsive mode or deploy and test on actual devices
```

## Known Issues

- **Initial implementation**: No bounds checking for extreme pan positions
- **Performance**: Very rapid gestures may cause lag on slower devices
- **Accessibility**: Limited keyboard/screen reader support for zoom controls

## Files Modified

- `src/components/tei_viewer.rs`: Main implementation
- Added web_sys imports: `PointerEvent`, `WheelEvent`
- Enhanced image container with touch event handling

## Commit History

- ✅ Added pointer event message types
- ✅ Implemented multi-touch tracking state
- ✅ Added pointer event handlers with pan/zoom logic
- ✅ Integrated pointer capture for robust tracking
- ✅ Added CSS touch-action prevention
- ✅ Fixed type consistency issues
- ✅ Added zoom center calculation for natural pinch behavior