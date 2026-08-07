# Fenestra

Fenestra is a native UI framework written in Rust for Windows and Linux. It uses HTML-like components and CSS-like styling, but does not embed a browser, WebView, or JavaScript runtime.

The framework provides its own UI tree, layout, styling, rendering, and native window integration. It is intended for normal application interfaces as well as transparent windows, overlays, notifications, screen capture, and other things that web runtimes tend to make unnecessarily awkward.

The idea is to keep the useful parts of writing web interfaces without actually shipping a web browser with the application.
