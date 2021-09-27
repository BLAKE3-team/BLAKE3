{
  inputs = {
    utils = {
      url = github:yatima-inc/nix-utils;
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    nixpkgs.url = github:nixos/nixpkgs/nixos-21.05;
    flake-utils = {
      url = github:numtide/flake-utils;
      inputs.nixpkgs.follows = "nixpkgs";
    };

  };

  outputs =
    { self
    , utils
    , flake-utils
    , nixpkgs
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      lib = utils.lib.${system};
      pkgs = import nixpkgs { inherit system; };
      inherit (lib) buildRustProject testRustProject rustDefault filterRustProject;
      reference_impl = buildRustProject {
        name = "reference_impl";
        root = ./reference_impl;
      };
      rust = rustDefault;
      crateName = "BLAKE3";
      root = ./.;
      cLib = import ./c/default.nix { inherit pkgs system; };
    in
    {
      packages = {
        "${crateName}-rs" = buildRustProject { inherit root; };
        "${crateName}-c" = cLib;
      };

      checks = {
        "${crateName}-rs" = testRustProject { inherit root; };
      };

      # `nix develop`
      devShell = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.packages.${system};
        nativeBuildInputs = [ rust ];
        buildInputs = with pkgs; [
          rust-analyzer
          clippy
          rustfmt
        ];
      };
    });
}
