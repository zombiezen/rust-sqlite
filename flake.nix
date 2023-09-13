{
  description = "zombiezen-sqlite Rust crate";

  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.callPackage ./package.nix {};

        checks.cargoTestModern = self.packages.${system}.default.overrideAttrs {
          cargoBuildType = "debug";
          cargoTestType = "debug";
        };

        checks.cargoTestNoModern = self.packages.${system}.default.overrideAttrs {
          cargoBuildType = "debug";
          cargoTestType = "debug";
          cargoBuildNoDefaultFeatures = true;
          cargoCheckNoDefaultFeatures = true;
        };

        checks.cargoTestBuildtimeBindgen = self.packages.${system}.default.overrideAttrs {
          cargoBuildType = "debug";
          cargoTestType = "debug";
          cargoBuildFeatures = [ "buildtime_bindgen" ];
          cargoTestFeatures = [ "buildtime_bindgen" ];
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];

          packages = [
            pkgs.cargo
            pkgs.rust-analyzer
            pkgs.rustfmt
          ];
        };
      }
    );
}
