{ nixpkgs ? import <nixpkgs> { } }:
let
  inherit (nixpkgs) pkgs;
#  project = pkgs.callPackage ./derivation.nix { };
in
pkgs.stdenv.mkDerivation {
  name = "capi";
  src = ./.;

  buildInputs = with pkgs; [ openssl ];
  nativeBuildInputs = with pkgs; [ cmake ];
}
