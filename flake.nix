{
  description = "Structure-aware source outline and context tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
      ];
      eachSystem = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = eachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          lib = pkgs.lib;
          manifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        in
        rec {
          lupa = pkgs.rustPlatform.buildRustPackage {
            pname = manifest.package.name;
            version = manifest.package.version;

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            meta = {
              description = "Structure-aware source outline and context tool";
              homepage = "https://github.com/0xdeafbeef/lupa";
              license = lib.licenses.wtfpl;
              mainProgram = "lupa";
              platforms = lib.platforms.linux;
            };
          };

          default = lupa;
        }
      );

      apps = eachSystem (
        system:
        rec {
          lupa = {
            type = "app";
            program = "${self.packages.${system}.lupa}/bin/lupa";
            meta.description = "Run lupa";
          };

          default = lupa;
        }
      );

      checks = eachSystem (system: {
        default = self.packages.${system}.lupa;
      });
    };
}
