{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      naersk,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        defaultPackage = (naersk-lib.buildPackage ./.).overrideAttrs (
          o:
          let
            yt-dlp = pkgs.yt-dlp.overrideAttrs (o: {
              version = "2024.10.16.232911.dev0";
            });
          in
          {
            buildInputs = o.buildInputs ++ [ pkgs.makeWrapper ];
            postFixup = ''
              wrapProgram $out/bin/playlist \
              --set PATH ${
                nixpkgs.lib.makeBinPath [
                  yt-dlp
                  pkgs.ffmpeg
                ]
              }
            '';
          }
        );
        devShell =
          with pkgs;
          mkShell {
            buildInputs = [
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
              cargo-watch
            ] ++ (if stdenv.isDarwin then [ libiconv ] else [ ]);
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      }
    );
}
