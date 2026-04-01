{ lib, ... }:
{
  perSystem =
    { self', pkgs, ... }:
    let
      inherit (pkgs) rustPlatform;
      fs = lib.fileset;
      cargoToml = fromTOML (builtins.readFile ../Cargo.toml);
    in
    {
      treefmt.programs.rustfmt.enable = true;

      devShells.default = pkgs.mkShell {
        inputsFrom = [
          self'.packages.default
        ];
        packages = [
          pkgs.clippy
          pkgs.rust-analyzer
        ];
      };

      packages.default = rustPlatform.buildRustPackage (finalAttrs: {
        pname = cargoToml.package.name;
        version = cargoToml.package.version;

        src = fs.toSource {
          root = ../.;
          fileset = fs.unions [
            ../src
            ../Cargo.toml
            ../Cargo.lock
          ];
        };

        nativeCheckInputs = [
          pkgs.clippy

          # Tools used by `examples/git-treeport.toml`, which is used in our tests.
          pkgs.git
          pkgs.nushell
        ];

        checkPhase = ''
          runHook preCheck
          cargo clippy --all-targets --all-features -- -D warnings
          runHook postCheck
        '';

        cargoLock.lockFile = ../Cargo.lock;
      });

      checks.package = self'.packages.default;
    };
}
