# whereami
## Features

- Navigate windows with arrow keys and mouse
- See which workspace each window is on
- Shows window state (tiled, floating, fullscreen, maximized)
- Configurable theming via TOML


## Requirements
- Hyprland or Niri
- Rust (for building)

## Install

**On NixOS**

```nix
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

// in niri config.kdl
binds {
    Mod+D { spawn "whereami"; }
}

// For hyprland
bind = SUPER, D, exec, whereami
```

- Arrow keys up/down = navigate (or use the mouse!)
- Escape = exit (if you are writing on the search bar, it will be two clicks to escape)
- Enter/Left click = focus selected window
- DEL/Right click = close selected window (unfortunately DEL does not work when typing, so press ESC first then press DEL)

## Configuration
- found in $HOME/.config/whereami/config.toml (auto-generated on first run)
- basic customisation provided (for now, some are not implemented)


# TODO
- [x] Added Niri Functionality
- [x] Change to iced layershell
- [ ] Remove unwraps
- [x] Rework flake.nix to be more "standard" - I dont know about standard but it's way better now



