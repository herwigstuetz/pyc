{}:
let
  nixpkgs = import <nixpkgs> { };
  pkgs = nixpkgs.pkgs;

  project = import ./default.nix { };
in
pkgs.stdenv.mkDerivation {
  name = "cmake-shell";

  nativeBuildInputs = project.nativeBuildInputs;

  buildInputs = with pkgs; [
    cmake
    ccls
  ] ++ project.buildInputs;
}
