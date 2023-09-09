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
      in {
        packages.default = nixpkgs.legacyPackages.${system}.callPackage ./default.nix {};
        formatter = nixpkgs.legacyPackages.${system}.alejandra;
      }
      ) // {
        nixosModules = let
          config = import ./service.nix {
            inherit nixpkgs;
          };
          pkgs = import nixpkgs {
            system = "aarch64-linux";
          };
          lib = import nixpkgs.lib;
        in{
          hydrophonitor-gps = import ./service.nix {
            inherit config pkgs lib;
          };
        };
      };
}
