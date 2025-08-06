{
  description = "Rematch bot";
  inputs.nixpkgs.url = "nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      defaultPackage.${system} = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustfmt
          rustc
          clippy
          pkg-config
          openssl
          cmake
          docker
          just
          chromium
          libopus
          just
          yamlfmt
        ];

        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        CHROME_BIN = "${pkgs.chromium}/bin/chromium";
      };

    };

}

