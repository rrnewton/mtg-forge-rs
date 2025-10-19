

This contains encodings of the magic rules in various lengths. I'm
trying to get a clean, compact summarized version for briefing LLMs in
the rules. 



Prompts
================================================================================

Create a Markdown translation of this PDF document. You can keep the
section structure and most of the text the same. Tables can be
translated to markdown tables. Section references should be updated to
refer to specific sections (using intra-document links to anchor tags
based on section headers), rather than page numbers in the original PDF.

The challenge will be coming up with textual translations of the
graphical figures.  Cards can be translated into text figures,
probably indented four spaces in code mode, for example:

    Messenger Drake {3}{U}{U}
    Creature -- Drake
    Flying
    When Messenger Drake dies, draw a card.
    3/3

I.e. you can use standard notation for Mana: {W}, {U}, {B}, {R}, {G},
and {C}. For showing a battlefield you should essentially show a read
out of the cards in each zone (by name), with an indication of
"(tapped)", life totals, etc.

----------------------------------------

This is the full official MTG rule book, with over 157,000 words.
Let's create a summary which is roughly 5-10 times shorter, at around
15,000 words or 30 pages instead of 300. There are a few ways we can
simplify (and make a note of what is omitted). We don't need to worry
about non-standard game formats or multiplayer games. Just the basic,
60+ card deck 1v1 game is the focus. We don't need to maintain every
obscure ruling around corner case interactions, but rather the idea is
to catch the most important underlying concepts of how to play the
game and how it works.

Please create a markdown file with correct intra-section links (using
section header anchors), or, for external sources use proper citations
with a URL. You can use an informal citation format for citing the
section numbers in the original rules document, e.g. "(Rules 100.6b)",
just explain at the outset what those references mean.

When I asked you this before without Deep Research mode you generated
a 2436 word summary. I am hoping with more planning you can generate a
summary in the 10,000-30,000 word range that captures more of the
originaly information.


