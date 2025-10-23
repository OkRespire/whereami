# whereami

A simple process viewer for Hyprland

## Features

- Focus a window by clicking on it
- Sort windows by workspace
- Quit the window manager
- Find where processes are.



## Install

**On NixOS**

```bash
$ git clone https://github.com/OkRespire/whereami.git
$ cd whereami
$ nix profile install

```
**This will later be written in flake format**



**Every other system**

Follow first two commands of the NixOS portion then

```bash
$ cargo build --release
$ cd target/release
$ ./whereami
```


*I will try to release it on package managers, maybe*
