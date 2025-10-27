# whereami
## Features

- Navigate windows with arrow keys
- See which workspace each window is on
- Shows window state (tiled, floating, fullscreen, maximized)
- Configurable theming via TOML


## Requirements
- Hyprland
- Rust (for building)

## Install

**On NixOS**

```bash
$ git clone https://github.com/OkRespire/whereami.git
$ cd whereami
$ nix profile install

```
**This will later be written in flake format**



**Every other system (Linux)**

**Other systems**
```bash
git clone https://github.com/OkRespire/whereami.git
cd whereami
cargo install --path .
```

## Usage
- Launch with 'whereami' in the terminal, or bind in your Hyprland config
```
bind = SUPER, D, exec, whereami

# for window rules
windowrulev2 = float, title:(whereami)
windowrulev2 = center, title:(whereami)
```

- Arrow keys up/down = navigate
- Escape = exit
- Enter = focus selected window

## Configuration
- found in $HOME/.config/whereami/config.toml (auto-generated on first run)
- basic customisation provided (for now, some are not implemented)

*I will try to release it on package managers, maybe*
