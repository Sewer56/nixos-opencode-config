# OpenCode Config

Personal OpenCode Home Manager config used in [Sewer56/nixos-setup](https://github.com/Sewer56/nixos-setup). 

Not intended for public use; not portable. Contains hardcoded paths, references to secrets etc.
Feel free to study it though.

I split this off from my main system repo to keep history isolated, because I update this often enough that it makes it hard to track OS/system changes.

## Building OpenCode

I use local self-built copy of OpenCode, which lives in `/home/sewer/Project/opencode/packages/opencode` on my system.

Build can be done by invoking `opencode-build`.
Which can then be used via `opencode` in the CLI.