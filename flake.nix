{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
  }:
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
        naersk-lib = pkgs.callPackage naersk {};
      in {
        defaultPackage = (naersk-lib.buildPackage ./.).overrideAttrs (
          o: let
            # yt-dlp = pkgs.yt-dlp.overrideAttrs (o: {
            #   version = "2025.1.26 ";
            # });
          in {
            buildInputs = o.buildInputs ++ [pkgs.makeWrapper];
            postFixup = ''
              wrapProgram $out/bin/playlist \
              --set PATH ${
                nixpkgs.lib.makeBinPath [
                  pkgs.yt-dlp
                  pkgs.ffmpeg
                ]
              }
            '';
          }
        );
        devShell = with pkgs;
          mkShell {
            packages =
              [
                cargo
                rust-analyzer
                rustc
                rustfmt
                pre-commit
                openssl
                rustPackages.clippy
                cargo-watch
                pkg-config
                ffmpeg.dev
              ]
              ++ (
                if stdenv.isDarwin
                then [libiconv]
                else []
              );
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      }
    );
}
