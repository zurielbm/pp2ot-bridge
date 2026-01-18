## [0.1.3] - 2026-01-18

### 🚀 Features

- Redesign the application UI with an industrial control panel theme, introducing new components for status, control, console, and settings, while also adding new documentation and updating dependencies.
- Implement application settings management, ProPresenter playlist fetching, and display fetched playlist items in the UI.
- Implement a time formatter page for organizing playlist items into timed groups and update PP host/port settings.
- Streamline UI by removing the home dashboard and promoting the formatter, add Ontime data structures, and introduce a duration formatting helper.
- Implement explicit insertion points using "Reference" items to control where new entries are added in the formatter, replacing the old dropdown selector.
- Dynamically calculate new entry end times based on preceding reference items and add a `link_start` property to timed entries.
- Add a new time picker component with corresponding styling and utility classes.
- Use settings for default duration, add a new favorite end time, simplify end time assignment logic, and remove unused Tailwind CSS classes.
- Implement drag-and-drop reordering for formatter items and add a `cargo check` agent rule.
- Implement a collapsible live logs panel, adjust the window's minimum size, and enhance app container scrolling with new flex utilities.

### 🐛 Bug Fixes

- Prevent duplicate formatter items and references, and add default transition CSS variables.
- Integrate Tailwind CSS by configuring Dioxus.toml and adding generated utility classes to `assets/tailwind.css`.
- Use user config directory for settings (fixes save failure in production)

### 💼 Other

- Added icon and other info
- Cargo update
- Cargo update
- Removed cargo edit

### 🚜 Refactor

- Unify formatter item management with a new `FormatterItem` enum, robustify Ontime API parsing, and remove unused CSS transition variables.
- Extract application settings, UI components, and data types into dedicated modules.

### 📚 Documentation

- Introduce documentation for Ontime event, project, and runtime data, add a default CSV, and update application logic, settings, and styling.

### ⚙️ Miscellaneous Tasks

- Update release and dioxus configs
