{
  description = "GPS recorder for Hydrophonitor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }: let
    systems = with flake-utils.lib.system; [
      x86_64-linux
      aarch64-linux
    ];
  in
    flake-utils.lib.eachSystem systems (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };
        service = import ./service.nix {
          inherit pkgs;
        };
      in {
        packages.default = nixpkgs.legacyPackages.${system}.callPackage ./default.nix {};
        formatter = nixpkgs.legacyPackages.${system}.alejandra;
      }
    );
}
