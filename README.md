# nora

Freezes the screen then runs a program and unfreezes the screen again. The main
goal is to change how screenshot tools behave when the screen updates.

## Examples

```bash
# Running a simple screenshot tool
nora maim -s image.png

# Running a bash command
nora bash -c 'shotgun -g $(hacksaw)'
```

## Installing

On Arch Linux the AUR package `nora` can be used.

Anywhere else binaries can be found under the
[releases](https://github.com/Jokler/nora/releases) section or
`cargo install nora` can be used to install nora through cargo.

This repo is also a flake, you can install it on NixOS through the overlay or as
a package.

```nix
{
  inputs = {
    nora.url = "github:Jokler/nora";
    # ...
  };
  outputs = {
    nora,
    ...
  }: {
    nixosConfigurations = {
      host = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          # ...
          ({pkgs, ...}: {
            # via overlay
            nixpkgs.overlays = [ nora.overlays.default ];
            environment.systemPackages = [ pkgs.nora ];

            # or via package:
            environment.systemPackages = [ nora.packages.x86_64-linux.nora ];
          })
        ];
      };
    };
  };
}
```
