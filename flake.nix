{
  description = "A no_std staticlib";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    naersk.url = "github:nmattia/naersk";
    mozilla = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, naersk, mozilla }:

    let system = "x86_64-darwin";

        rust = (pkgs.rustChannelOf {
#          date = "2021-05-08";
          channel = "nightly";
          sha256 = "sha256-3PttEUITkS/+W/3jJXW0iUXDF3TMcPMUo1agySr6rpk=";
        }).rust;

        rustOverlay = final: prev: {
          rustc = rust;
          cargo = rust;
        };

        # Override the version used in naersk
        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };

        pkgs = import nixpkgs {
          inherit system;
          config = { };
          overlays = [
            (import mozilla) # for rustChannelOf
            rustOverlay # for rustc and cargo of the specific channel
            naersk.overlay # for buildPackage
          ];
        };

    in {
      packages.${system}.minimal = naersk-lib.buildPackage {
        src = ./.;

        buildInputs = with pkgs; [
          # for cbindgen
          libiconv

          # pyo3
          python38
        ];
      };

      defaultPackage.${system} = self.packages.${system}.minimal;

      devShell.${system} = pkgs.mkShell {
        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        buildInputs = with pkgs; [
          cargo
          clippy
          rustc
          rust-analyzer
          rustfmt

          python38
        ];
      };
    };
}
