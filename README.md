# nora

Freezes the screen then runs a program and unfreezes the screen again.
The main goal is to change how screenshot tools behave when the screen updates.

## Examples
```bash
# Running a simple screenshot tool
nora maim -s image.png

# Running a bash command
nora bash -c 'shotgun -g $(hacksaw)'
```

## Installing
On Arch Linux the AUR package `nora` can be used.

On NetBSD `nora` is available through the main pkgsrc repository thanks to 0323pin.

Anywhere else binaries can be found under the [releases](https://github.com/Jokler/nora/releases)
section or `cargo install nora` can be used to install nora through cargo.
