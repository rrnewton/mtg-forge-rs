random choices of 1 option still present.
----------------------------------------

Look at an e2e game: 

```
$ cargo run --release --bin mtg -- tui test_decks/simple_bolt.dck test_decks/grizzly_bears.dck --p1=random --p2=random --seed=41 -v2

>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose creature 0 to attack (50% probability) out of 3 available creatures
>>> RANDOM chose creature 2 to attack (50% probability) out of 3 available creatures
  Player 2 declares Grizzly Bears (85) (2/2) as attacker
  Player 2 declares Grizzly Bears (83) (2/2) as attacker
  Grizzly Bears (83) deals 2 damage to Player 1
  Grizzly Bears (85) deals 2 damage to Player 1
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
```

I still see all these choices where the ONLY option is pass
priority. These should NOT be logged as a choice, but should be
suppressed (the player should not be asked the choice).

Our guiding principle here is to only invoke the PlayerController 
 - when a choice is needed
 - presenting VALID options only

When `get_available_spell_abilities` returns only a single action
(PassPriority), then we don't need to call the PlayerController.

