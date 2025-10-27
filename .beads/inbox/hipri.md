

Push through support for `monored.dck` deck
-------------------------------------------
What it means to make a at all usable is:
- the cards load
- random play works

But to FULLY support a deck and make it playable through the TUI:
- mechanics on all cards work
- the heuristic AI can play it somewhat reasonably
- all choices are presented correctly to the controller

You can use all the tools at your disposal to do this.
- prefer E2E experiments with the actual mtg binary, it's easier
  for us to be mislead about what works when not using the full `tui`


TODO: Guide on how to make general progress on the TUI
--------------------------------------------------------------------------------

### General guide

Now you have the tools you need to:
 - play real games of magic with real decks
 - have the full experience that I will have while playing (stop/resume games to explore particular choice points)
 - identify gaps/bugs and make progress to fix them.

Indeed, you should prefer using the `mtg tui` CLI directly for testing where
possible, because that will ensure you're testing the same experience I'm
viewing. Unit testing is fine for targeted checking of functionality, but we
need heavy e2e testing.




### Specific Instructions


### What to do with this task


### Tracking area

