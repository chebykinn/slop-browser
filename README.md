# Slop Browser

A minimal web browser written in Rust using SDL2 for windowing and wgpu (WebGPU) for GPU-accelerated rendering. Implements HTML5 parsing, CSS cascade/specificity, multiple layout algorithms, and a basic JavaScript interpreter.

## Overview

Slop Browser is a from-scratch browser engine implementation featuring:

- **HTML5 Parsing** - Full spec compliance via `html5ever`
- **CSS Engine** - Cascade, specificity, selector matching via `cssparser`
- **Layout Engine** - Block, inline, flexbox, grid, and table layout algorithms
- **GPU Rendering** - WebGPU-based rendering with `wgpu`
- **Text Rendering** - Modern GPU text via `glyphon` and `cosmic-text`
- **Async Networking** - Non-blocking HTTP with progress tracking via `reqwest`/`tokio`
- **JavaScript** - Custom interpreter with DOM bindings
- **SVG Support** - Vector graphics via `resvg`

~14,500 lines of Rust code across modular components.

## Building

### Prerequisites

- Rust 1.70+ (Edition 2021)
- SDL2 development libraries
- GPU with Vulkan, Metal, or DX12 support

#### Debian/Ubuntu
```bash
sudo apt install libsdl2-dev libsdl2-ttf-dev
```

#### macOS
```bash
brew install sdl2
```

#### Arch Linux
```bash
sudo pacman -S sdl2
```

### Build

```bash
# Debug build
cargo build

# Release build (recommended for performance)
cargo build --release

# Run
cargo run --release -- --url https://example.com
```

### CLI Arguments

| Flag | Description |
|------|-------------|
| `--url <URL>` | URL to load on startup |
| `--js` / `--no-js` | Enable/disable JavaScript execution |
| `--css` / `--no-css` | Enable/disable CSS styling |
| `--screenshot <PATH>` | Headless mode: render to PNG file |
| `--width <N>` | Screenshot width (default: 1024) |
| `--height <N>` | Screenshot height (default: 768) |
| `--debug` | Print DOM tree, layout tree, and display list |

**Examples:**
```bash
# Interactive browsing
cargo run --release -- --url https://news.ycombinator.com

# Headless screenshot
cargo run --release -- --url https://example.com --screenshot out.png --width 1920 --height 1080

# Debug mode
cargo run --release -- --url https://example.com --debug
```

## Technical Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         main.rs                                  │
│                    (SDL2 Event Loop)                             │
└─────────────────────────┬───────────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────────┐
│                        Browser                                   │
│              (Central State Management)                          │
│  ┌──────────┬──────────┬──────────┬──────────┬───────────────┐  │
│  │   Tab[]  │  Chrome  │ Loader   │AsyncLoader│   Runtime    │  │
│  └────┬─────┴────┬─────┴────┬─────┴─────┬────┴───────────────┘  │
└───────┼──────────┼──────────┼───────────┼───────────────────────┘
        │          │          │           │
┌───────▼──────┐ ┌─▼────────┐ │     ┌─────▼─────┐
│     DOM      │ │    UI    │ │     │  Network  │
│  ┌────────┐  │ │ ┌──────┐ │ │     │ ┌───────┐ │
│  │Document│  │ │ │Chrome│ │ │     │ │reqwest│ │
│  │ Node   │  │ │ └──────┘ │ │     │ │ tokio │ │
│  │ Parser │  │ └──────────┘ │     │ └───────┘ │
│  └────────┘  │              │     └───────────┘
└───────┬──────┘              │
        │                     │
┌───────▼──────┐        ┌─────▼─────┐
│     CSS      │        │  Layout   │
│ ┌──────────┐ │        │ ┌───────┐ │
│ │StyleComp │ │        │ │ Block │ │
│ │Stylesheet│ │───────▶│ │Inline │ │
│ │ Cascade  │ │        │ │ Flex  │ │
│ │Selector  │ │        │ │ Grid  │ │
│ └──────────┘ │        │ │ Table │ │
└──────────────┘        │ └───────┘ │
                        └─────┬─────┘
                              │
                        ┌─────▼─────┐        ┌───────────┐
                        │  Render   │        │    JS     │
                        │ ┌───────┐ │        │ ┌───────┐ │
                        │ │Painter│ │        │ │Interp │ │
                        │ │  GPU  │─┼───────▶│ │ DOM   │ │
                        │ │ Text  │ │        │ │Binding│ │
                        │ │ Image │ │        │ └───────┘ │
                        │ └───────┘ │        └───────────┘
                        └───────────┘
```

### Module Structure

```
src/
├── main.rs              # Entry point, event loop, CLI
├── lib.rs               # Library exports
├── app/
│   ├── browser.rs       # Browser state, navigation, rendering orchestration
│   ├── tab.rs           # Tab: DOM + stylesheets + layout + history
│   ├── history.rs       # Back/forward navigation stack
│   └── settings.rs      # Browser configuration
├── dom/
│   ├── parser.rs        # HTML parsing (html5ever integration)
│   ├── document.rs      # Document structure
│   └── node.rs          # DOM node types and traversal
├── css/
│   ├── parser.rs        # CSS parsing (cssparser integration)
│   ├── cascade.rs       # Cascade algorithm, specificity calculation
│   ├── computed.rs      # ComputedStyle: resolved property values
│   ├── selector.rs      # Selector parsing and matching
│   ├── stylesheet.rs    # Stylesheet and rule representation
│   └── index.rs         # Selector indexing for fast matching
├── layout/
│   ├── tree.rs          # LayoutTree construction and management
│   ├── box_model.rs     # CSS box model (content/padding/border/margin)
│   ├── block.rs         # Block formatting context
│   ├── inline.rs        # Inline formatting context
│   ├── flex.rs          # Flexbox layout algorithm
│   ├── grid.rs          # CSS Grid layout
│   ├── table.rs         # Table layout algorithm
│   └── text.rs          # Text measurement and line breaking
├── render/
│   ├── gpu.rs           # wgpu device/queue/surface setup
│   ├── painter.rs       # Display list generation and execution
│   ├── text.rs          # Text rendering (glyphon integration)
│   ├── image_cache.rs   # Image loading, decoding, caching
│   ├── texture.rs       # GPU texture management
│   └── shaders/         # WGSL shader programs
├── net/
│   ├── async_loader.rs  # Async HTTP with progress/cancellation
│   ├── loader.rs        # Synchronous HTTP loader
│   ├── http.rs          # HTTP utilities
│   └── cache.rs         # Resource caching
├── js/
│   ├── interpreter.rs   # JavaScript execution engine
│   ├── parser.rs        # JS AST parser
│   ├── lexer.rs         # JS tokenizer
│   └── dom_bindings.rs  # document.*, console.* bindings
├── input/
│   ├── keyboard.rs      # Keyboard event handling
│   ├── mouse.rs         # Mouse event handling
│   └── events.rs        # Input event types
└── ui/
    └── chrome.rs        # Browser chrome (address bar, buttons)
```

### Data Flow

#### Page Load Pipeline

```
URL
 │
 ▼
┌──────────────────┐
│   AsyncLoader    │  HTTP request via reqwest
│  (tokio runtime) │  Progress events → mpsc channel
└────────┬─────────┘
         │ HTML bytes
         ▼
┌──────────────────┐
│   html5ever      │  Tokenize → TreeBuilder
│   DOM Parser     │  Handles malformed HTML
└────────┬─────────┘
         │ DOM tree
         ▼
┌──────────────────┐
│  CSS Extraction  │  <style>, <link rel="stylesheet">
│  & Parsing       │  Inline styles
└────────┬─────────┘
         │ Stylesheet[]
         ▼
┌──────────────────┐
│  Style Computer  │  Cascade: user-agent < author < inline
│  (Cascade)       │  Specificity: (inline, id, class, type)
└────────┬─────────┘
         │ ComputedStyle per element
         ▼
┌──────────────────┐
│  Layout Engine   │  Block → child positioning
│  (Box Model)     │  Inline → line box wrapping
│                  │  Flex → main/cross axis distribution
└────────┬─────────┘
         │ LayoutTree with positions/sizes
         ▼
┌──────────────────┐
│  Display List    │  SolidRect, Text, Image, Border commands
│  Generation      │  Clipping, z-ordering
└────────┬─────────┘
         │ DisplayList
         ▼
┌──────────────────┐
│   GPU Painter    │  wgpu render pass
│   (wgpu)         │  Vertex buffers, texture sampling
└────────┬─────────┘
         │
         ▼
      Screen
```

#### Rendering Pipeline

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ DisplayList │───▶│   Painter   │───▶│   Shaders   │
│  Commands   │    │  (Batching) │    │   (WGSL)    │
└─────────────┘    └─────────────┘    └─────────────┘
                          │
         ┌────────────────┼────────────────┐
         ▼                ▼                ▼
   ┌───────────┐   ┌───────────┐   ┌───────────┐
   │ SolidRect │   │   Text    │   │   Image   │
   │  Quads    │   │  Glyphon  │   │ Textures  │
   └───────────┘   └───────────┘   └───────────┘
```

### Key Data Structures

#### Browser State
```rust
pub struct Browser {
    tabs: Vec<Tab>,
    active_tab: usize,
    chrome: Chrome,
    loader: Loader,
    async_loader: Option<AsyncLoader>,
    runtime: Runtime,                    // tokio
    progress_receiver: Option<Receiver<LoadProgress>>,
    display_list_cache: Option<DisplayList>,
    settings: BrowserSettings,
}
```

#### Tab
```rust
pub struct Tab {
    pub document: Option<Document>,
    pub stylesheets: Vec<Stylesheet>,
    pub layout_tree: Option<LayoutTree>,
    pub history: History,
    pub loading_state: LoadingState,
    pub error: Option<String>,
    pub progress: f32,
    pub pending_images: HashSet<String>,
    pub image_cache: ImageCache,
}
```

#### ComputedStyle
```rust
pub struct ComputedStyle {
    pub display: Display,           // Block, Inline, Flex, Grid, None
    pub position: Position,         // Static, Relative, Absolute, Fixed
    pub color: Color,
    pub background_color: Color,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub width: Size,
    pub height: Size,
    pub margin: Edges,
    pub padding: Edges,
    pub border: BorderStyle,
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    // ... 40+ properties
}
```

#### LayoutBox
```rust
pub struct LayoutBox {
    pub box_type: BoxType,          // Block, Inline, InlineBlock, Anonymous
    pub style: ComputedStyle,
    pub content_rect: Rect,         // Content area
    pub padding_rect: Rect,         // Content + padding
    pub border_rect: Rect,          // Content + padding + border
    pub margin_rect: Rect,          // Full box
    pub children: Vec<LayoutBox>,
    pub scroll_offset: f32,
    pub scrollbar_state: ScrollbarState,
}
```

#### Display Commands
```rust
pub enum DisplayCommand {
    SolidRect { rect: Rect, color: Color },
    Text { text: String, position: Point, style: TextStyle },
    Image { rect: Rect, texture_id: TextureId },
    Border { rect: Rect, style: BorderStyle },
    PushClip { rect: Rect },
    PopClip,
}
```

### CSS Specificity

The cascade follows standard CSS specificity rules:

| Selector Type | Specificity |
|--------------|-------------|
| Inline style | (1, 0, 0, 0) |
| ID selector `#id` | (0, 1, 0, 0) |
| Class `.class`, attribute `[attr]`, pseudo-class `:hover` | (0, 0, 1, 0) |
| Type `div`, pseudo-element `::before` | (0, 0, 0, 1) |

Implemented in `css/cascade.rs` with indexed selector matching for performance.

### Layout Algorithms

#### Block Layout (`layout/block.rs`)
- Vertical stacking of block-level children
- Width: constrained by containing block
- Height: sum of children heights + margins
- Margin collapsing between adjacent blocks

#### Inline Layout (`layout/inline.rs`)
- Horizontal text flow with line wrapping
- Line boxes contain inline elements
- Vertical alignment within line boxes
- Text measurement via cosmic-text

#### Flexbox (`layout/flex.rs`)
- Main axis / cross axis distribution
- `flex-grow`, `flex-shrink`, `flex-basis`
- `justify-content`, `align-items`, `align-self`
- Wrapping with `flex-wrap`

#### Grid (`layout/grid.rs`)
- Track sizing (fixed, fractional, auto)
- `grid-template-columns`, `grid-template-rows`
- Item placement

#### Table (`layout/table.rs`)
- Column width calculation
- Row height distribution
- Cell spanning

### GPU Rendering

The renderer uses wgpu with the following setup:

```rust
// Device selection (prefers discrete GPU)
let adapter = instance.request_adapter(&RequestAdapterOptions {
    power_preference: PowerPreference::HighPerformance,
    compatible_surface: Some(&surface),
    ..
}).await;

// Surface configuration
surface.configure(&device, &SurfaceConfiguration {
    usage: TextureUsages::RENDER_ATTACHMENT,
    format: TextureFormat::Bgra8UnormSrgb,
    present_mode: PresentMode::Fifo,
    ..
});
```

Text rendering uses glyphon's `TextRenderer` with cosmic-text for shaping:

```rust
// Text buffer preparation
let mut buffer = Buffer::new(&mut font_system, Metrics::new(font_size, line_height));
buffer.set_text(&mut font_system, text, Attrs::new(), Shaping::Advanced);

// GPU rendering
text_renderer.prepare(&device, &queue, &mut font_system, &mut atlas, ...).unwrap();
text_renderer.render(&atlas, &mut pass).unwrap();
```

### Async Loading

Network requests run on a dedicated tokio runtime to avoid blocking the UI:

```rust
pub struct AsyncLoader {
    runtime: Runtime,
    cancel_token: CancellationToken,
}

impl AsyncLoader {
    pub fn load(&self, url: &str, sender: Sender<LoadProgress>) {
        let token = self.cancel_token.clone();
        self.runtime.spawn(async move {
            let response = reqwest::get(url).await?;
            let total = response.content_length();
            let mut stream = response.bytes_stream();

            while let Some(chunk) = stream.next().await {
                if token.is_cancelled() { break; }
                sender.send(LoadProgress::Data(chunk?)).await?;
            }
            sender.send(LoadProgress::Complete).await?;
        });
    }
}
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `html5ever` | 0.27 | HTML5 parsing (WHATWG spec compliant) |
| `markup5ever_rcdom` | 0.3 | Reference-counted DOM |
| `cssparser` | 0.34 | W3C CSS parsing |
| `sdl2` | 0.37 | Cross-platform windowing/input |
| `wgpu` | 22.1 | WebGPU graphics abstraction |
| `glyphon` | 0.6 | GPU text rendering |
| `cosmic-text` | 0.12 | Text shaping/measurement |
| `reqwest` | 0.12 | Async HTTP client |
| `tokio` | 1.0 | Async runtime |
| `image` | 0.25 | Image format decoding |
| `resvg` | 0.44 | SVG rendering |
| `url` | 2.5 | URL parsing/resolution |
| `clap` | 4.0 | CLI argument parsing |

## Browser Controls

| Control | Action |
|---------|--------|
| `Ctrl+L` | Focus address bar |
| `Escape` | Stop loading / Exit |
| `Mouse wheel` | Scroll page |
| `Click` | Navigate links / UI interaction |

## Limitations

- No HTTPS certificate validation in some cases
- JavaScript support is basic (no full ES6+)
- No Web APIs (fetch, WebSocket, etc.)
- Limited form support
- No cookies persistence across sessions
- Single process architecture

## License

MIT
