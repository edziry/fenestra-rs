# Fenestra

Fenestra is a native UI framework written in Rust for Windows and Linux. It uses HTML-like components and typed CSS-inspired styling, but does not embed a browser, WebView, or JavaScript runtime.

The framework provides its own UI tree, layout, styling, rendering, and native window integration. It is intended for normal application interfaces as well as transparent windows, overlays, notifications, advanced 2D graphics, and safely exportable framework-owned surfaces.

The idea is to keep the useful parts of writing web interfaces without actually shipping a web browser with the application.

Native capture, audio, encoding, and transport are application or ecosystem concerns rather than Fenestra core responsibilities. The Cargo package family uses `fenestra-ui` and `fenestra-ui-*`; its pre-alpha bootstrap is active under the [initial implementation plan](docs/initial-implementation-plan.md).

Workspace packages follow the ratified [pre-1.0 versioning policy](docs/versioning-policy.md): versions use `MAJOR.MINOR.PATCH`, and an intentional pre-1.0 compatibility break advances `MINOR` without requiring a compatibility shim.
