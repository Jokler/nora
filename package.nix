{
  pkgs,
  rustPlatform,
  fetchFromGitHub,
}:
rustPlatform.buildRustPackage rec {
  pname = "nora";
  version = "0.2.0";

  src = fetchFromGitHub {
    owner = "Jokler";
    repo = "nora";
    rev = "v${version}";
    hash = "sha256-VROBfmGgsp6pyUjs8CWag5Ft18SC4+oA43hfSyMlOjU=";
  };

  cargoHash = "sha256-Pu2fnQxTCmjEoFxT7VaZ4N/xu1BK+EfWH5+GVSfg8mk=";

  nativeBuildInputs = [pkgs.pkg-config];
  buildInputs = with pkgs; [xorg.libX11 xorg.libXfixes];
}
