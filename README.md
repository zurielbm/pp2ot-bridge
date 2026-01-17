# PP2OT Bridge

A native desktop application that bridges **ProPresenter** and **Ontime** by extracting playlist cue times from ProPresenter and formatting them for import into Ontime timelines.

Built with [Dioxus 0.7](https://dioxuslabs.com/) using Rust, featuring a modern dark UI with Tailwind CSS.

---

## Features

- **ProPresenter Integration**: Connect to ProPresenter's API to fetch playlists and their timing data
- **Ontime Integration**: Push formatted events directly to Ontime's API
- **Time Formatter**: Convert and group playlist items with editable cue times
- **Live Console**: Real-time logging panel for debugging API calls
- **Settings Management**: Configure connection settings, defaults, and sync preferences

---

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started)

```bash
# Install the Dioxus CLI
curl -sSL http://dioxus.dev/install.sh | sh
```

### Running the Dev Server

```bash
dx serve
```

This starts the development server with hot-reload enabled. Tailwind CSS is automatically processed (Dioxus 0.7+).

To explicitly run as a desktop app:

```bash
dx serve --platform desktop
```

### Verify Code Compiles

```bash
cargo check
```

---

## Building for Production

Bundle the application for release:

```bash
dx bundle --platform desktop --release
```

The bundled application will be output to `target/dx/pp2ot-bridge/release/bundle/`.

---

## Project Structure

```
pp2ot-bridge/
├── assets/              # Static assets (CSS, icons, images)
├── src/
│   ├── main.rs          # App entry point and routing
│   ├── components/      # UI components
│   │   ├── formatter.rs # Main formatter page
│   │   ├── navbar.rs    # Navigation bar
│   │   ├── settings.rs  # Settings page
│   │   └── time_picker.rs
│   ├── types/           # Data structures and API types
│   └── utils.rs         # Utility functions
├── Cargo.toml           # Rust dependencies
├── Dioxus.toml          # Dioxus configuration
└── tailwind.css         # Tailwind input file
```

---

## Configuration

Settings are persisted in `settings.json` and include:

- **ProPresenter**: Host, port, and target playlist name
- **Ontime**: Host and port for the Ontime server
- **Defaults**: Default duration and end action for new items
- **Sync**: Auto-refresh and timeline position preferences

---

## Tailwind CSS

Automatic Tailwind is enabled. No manual setup required.

To customize, modify `Dioxus.toml`:

```toml
[application]
tailwind_input = "tailwind.css"
tailwind_output = "assets/tailwind.css"
```

For plugins or advanced customization, see the [Tailwind CLI docs](https://tailwindcss.com/docs/installation/tailwind-cli).
