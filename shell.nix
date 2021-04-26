/*
let
  mozilla-overlay-checkout = builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/8c007b60731c07dd7a052cce508de3bb1ae849b4.tar.gz;
  mozilla-overlay = import "${mozilla-overlay-checkout}/rust-overlay.nix";
  pkgs = import <nixpkgs> { overlays = [ mozilla-overlay ]; };
  channel = pkgs.rustChannelOf { date = "2021-03-05"; channel = "nightly"; };
in pkgs.mkShell {
  buildInputs = with pkgs; [
    channel.rust
    pkg-config
    alsaLib.dev
  ];
}
*/

with import <nixpkgs> {};
mkShell {
  buildInputs = with pkgs; [
    cargo
    pkg-config
    alsaLib.dev
  ];
}
