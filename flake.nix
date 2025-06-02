{
  description = "Nix develop flake for this project";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url  = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

	rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
      in
      {
        devShells.default = pkgs.mkShell rec {
          buildInputs = [
	    rustToolchain
            pkgs.pkg-config
            pkgs.eza
            pkgs.fd
            pkgs.rust-bin.beta.latest.default
            pkgs.libxkbcommon
            pkgs.libGL
            pkgs.wayland
	    pkgs.openssl
          ];

          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";

          shellHook = ''
            alias ls=eza
            alias find=fd
          '';
        };
      }
    );
}

