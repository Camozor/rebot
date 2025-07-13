{
  description = "Rematch bot";
  inputs.nixpkgs.url = "nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      defaultPackage.${system} = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ rustfmt clippy pkg-config openssl ];
        buildInputs = with pkgs; [ cargo rustc docker chromium just ];

        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        CHROME_BIN = "${pkgs.chromium}/bin/chromium";
      };

    };

}

