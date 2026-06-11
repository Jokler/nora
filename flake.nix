{
  description = "Screen Freezer";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = {
    self,
    nixpkgs,
  }: let
    supportedSystems = ["x86_64-linux"];
    forEachSystem = nixpkgs.lib.genAttrs supportedSystems;
    overlayList = [self.overlays.default];
    pkgsBySystem = forEachSystem (system:
      import nixpkgs {
        inherit system;
        overlays = overlayList;
      });
  in {
    overlays.default = final: prev: {nora = final.callPackage ./package.nix {};};

    packages = forEachSystem (system: {
      nora = pkgsBySystem.${system}.nora;
      default = pkgsBySystem.${system}.nora;
    });
  };
}
