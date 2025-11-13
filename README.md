# whereami
## Features

- Navigate windows with arrow keys and mouse
- See which workspace each window is on
- Shows window state (tiled, floating, fullscreen, maximized)
- Configurable theming via TOML


## Requirements
- Hyprland
- Rust (for building)

## Install

**On NixOS**

```bash
# in flake.nix

    whereami = {
      url = "github:okrespire/whereami";
    };

# in home manager/normal nix package
    packages = with pkgs; [
     #... other packages
     inputs.whereami.packages.${system}.whereami
    ];

```
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

windowrulev2 = float, class:(whereami)
windowrulev2 = center, class:(whereami)
windowrulev2 = pin, class:(whereami) # this is so if you change workspaces it follows you
```

- Arrow keys up/down = navigate (or use the mouse!)
- Escape = exit (if you are writing on the search bar, it will be two clicks to escape)
- Enter/Left click = focus selected window
- DEL/Right click = close selected window (unfortunately DEL does not work when typing, so press ESC first then press DEL)

## Configuration
- found in $HOME/.config/whereami/config.toml (auto-generated on first run)
- basic customisation provided (for now, some are not implemented)

*I will try to release it on package managers, maybe*
