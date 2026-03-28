# Mewski Bot

A Discord bot for organizing your CTF team's work during competitions.

## Commands

### `/ctf create <name>`

Creates a forum channel `ctf-<name>` under the current category with tags for challenge status (`unsolved`, `solved`) and categories (`forensics`, `network`, `rev`, `crypto`, `web`, `stego`, `log-analysis`, `misc`). Pins a **General** thread as the team's hub for that CTF.

### `/chal create <name> <category>`

Run from a CTF's General thread. Creates a challenge thread tagged with its category and `unsolved`.

### `/chal solve <flag>`

Run from a challenge thread. Swaps `unsolved` for `solved`, renames the thread with a checkmark, archives it, and announces the solve in General with the flag spoilered.

### `/summarize [messages]`

Summarizes key discoveries and findings from the current channel using Claude. Parses text and images. Defaults to the last 100 messages (max 500). Requires the `claude` CLI on the host.

## Setup

```sh
# Add your DISCORD_TOKEN to .env
echo "DISCORD_TOKEN=your_token_here" > .env
nix develop  # or install Rust nightly manually
cargo run
```
