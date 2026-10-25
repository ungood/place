{
  description = "Manage workplaces: directories of independent git repositories";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      # nixpkgs dropped x86_64-darwin in 26.11.
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      # The sandbox has no .git, so build.rs can't run git describe. Name the
      # commit instead, such as 0.0.0-abc1234.
      version = "0.0.0-${self.shortRev or self.dirtyShortRev or "unknown"}";

      placePackage =
        {
          lib,
          rustPlatform,
          git,
        }:
        rustPlatform.buildRustPackage {
          pname = "place";
          inherit version;
          src = lib.cleanSource self;
          cargoLock.lockFile = ./Cargo.lock;

          env.PLACE_VERSION = version;

          # The cucumber features run git against local bare repos, which
          # works in the sandbox.
          nativeCheckInputs = [ git ];

          meta = {
            description = "Manage workplaces: directories of independent git repositories";
            homepage = "https://github.com/ungood/place";
            license = lib.licenses.mit;
            mainProgram = "place";
          };
        };
    in
    {
      overlays.default = final: _prev: { place = final.callPackage placePackage { }; };

      packages = forAllSystems (pkgs: rec {
        place = pkgs.callPackage placePackage { };
        default = place;
      });
    };
}
