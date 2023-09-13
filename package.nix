{ rustPlatform
, lib
, nix-gitignore
, sqlite
, pkg-config
}:

let
  cargoContents = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  pname = cargoContents.package.name;
in rustPlatform.buildRustPackage {
  inherit pname;
  version = cargoContents.package.version;

  src = let
    root = ./.;
    patterns = nix-gitignore.withGitignoreFile extraIgnores root;
    extraIgnores = [ ".github" ".vscode" "*.nix" "flake.lock" ];
  in builtins.path {
    name = "${pname}-source";
    path = root;
    filter = nix-gitignore.gitignoreFilterPure (_: _: true) patterns root;
  };

  cargoLock.lockFile = ./Cargo.lock;

  buildInputs = [
    sqlite.dev
  ];

  nativeBuildInputs = [
    pkg-config
    rustPlatform.bindgenHook
  ];

  passthru = {
    inherit sqlite;
  };

  meta = {
    homepage = "https://github.com/zombiezen/rust-sqlite";
    maintainers = [ lib.maintainers.zombiezen ];
  };
}
