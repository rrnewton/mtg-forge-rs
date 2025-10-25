
Begin or progress work on simple interactive TUI
------------------------------------------------------------

The basic `mtg tui --p1=tui` will be very similar to the Java version. Instead
of making a random choice for every `Enter choice (0-N)` prompt, we will present
the choice to the user and wait for input on stdin.

Later we will add a fancy TUI with the ratatui library, so we should keep an eye
on how to make our Controller interface generic. It will present the battlefield
and the choices in a different way.
