# Scoring

**Scoring** is a Solana program enabling DAOs and blockchain-integrated games to
record a user's *score* on-chain. Unlike tokens, a *score* is non-transferable
and may only be changed by its owning program. The generic unit for a score is
*points*.

Supplementing the on-chain program are client libraries for querying a wallet's
scores, rendering them, and producing leaderboards. A command-line tool is also
provided to simplify registering mint accounts.

## Data Model

Like standard [SPL tokens](https://spl.solana.com/token), scoring uses a
"decorator account" pattern.

A game developer first creates the *mint account* for their points. This account
contains metadata for rendering the score in a wallet: an icon, a localizable
display name, and the game's URL. This account *must* be rent-exempt.

To grant the user wallet a score, the game creates an *associated account* using
both the wallet and the mint's addresses. This account contains the actual
number of points for the wallet. This account's rent funding is decided by the
game developer.