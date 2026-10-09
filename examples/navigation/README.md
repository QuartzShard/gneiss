# Navigation

Two windows and the navigation between them (`src/main.rs`): a static home
window whose persistent data owns a detail window, click handlers that push and
pop through `Nav`, and a home `appear` that redraws from that data whenever the
window is back on top.

