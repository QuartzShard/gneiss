# Window navigation

Every kind of window, opened from one menu (`src/main.rs`).

- A static root `MenuWindow`, whose persistent data owns the windows it opens.
- A plain `Window`, pushed with a starting count through `UiQueue::push_with`.
  Up and Down repeat while held. The root menu counts how often it's opened,
  and holding Select on its row resets the count.
- A `SimpleMenuWindow` the firmware draws from a static table, in sections with
  headers.
- A `SimpleMenuWindow` of lap times only known at runtime, built once into a
  `MenuList`.
- A menu whose levels are all one `Menu` type. Each level opens the next, and
  frees it in its layout's `appear` once it's popped.
- A `ScrollWindow` reader sized to its text. On round screens, the text flows
  around the edge and scrolls a page at a time.

The stopwatch example covers the rest: an `ActionWindow` and a status bar.
