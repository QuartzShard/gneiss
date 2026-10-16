# Menus

Menu windows (`src/main.rs`): a static root menu whose persistent data owns the
menus it opens, a `SimpleMenuWindow` the firmware draws from a static table in
sections with headers, another whose rows are lap times only known at runtime,
built once into a `MenuList`, and a menu whose levels are all one `Menu` type, its rows
formatted from the level number into `StackText` as they're drawn, the first row
of each opening the next, and selecting an item eating it and queuing a reload
through `UiQueue`. Each menu frees the level it opened in its layout's `appear`,
once that level is popped. Selecting a fruit, or "About", pushes a plain window
with text to read.
