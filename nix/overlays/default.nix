final: prev: let
  cargo-tauri = prev.callPackage ./cargo-tauri.nix {};
in {
  inherit cargo-tauri;
}
