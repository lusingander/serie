{
  inputs.nixpkgs = {
    type = "github";
    owner = "NixOS";
    repo = "nixpkgs";
    ref = "nixos-unstable";
  };

  outputs = inputs:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = inputs.nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = inputs.nixpkgs.legacyPackages.${system};
        in
        {
          serie = pkgs.rustPlatform.buildRustPackage {
            pname = (pkgs.lib.importTOML (./Cargo.toml)).package.name;
            version = (pkgs.lib.importTOML (./Cargo.toml)).package.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            meta.mainProgram = "serie";
            nativeBuildInputs = [ pkgs.git ];
          };
          default = inputs.self.packages.${system}.serie;
        }
      );
      devShells = forAllSystems (
        system:
        let
          pkgs = inputs.nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            name = "devShell";
            packages = [
              inputs.self.packages.${system}.serie
              pkgs.rustfmt
            ];
          };
        }
      );
    };
}
