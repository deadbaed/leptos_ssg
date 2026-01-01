{
  sources ? import ./npins,
  pkgs ? import sources.nixpkgs { },
  lib ? pkgs.lib,
}:

let
  mkTailwindStylesheet =
    name: src: minify:
    pkgs.runCommand "${name}-compiled-tailwind"
      {
        inherit src;
      }
      ''
        mkdir $out
        ${pkgs.tailwindcss_4}/bin/tailwindcss --cwd $src ${lib.optionalString minify "--minify"} > $out/style.css
      '';
  tailwindOpengraph = "${mkTailwindStylesheet "opengraph" ./opengraph/src true}/style.css";
  tailwindLeptosSsg = "${mkTailwindStylesheet "leptos_ssg" ./src true}/style.css";
in
{
  shell = pkgs.mkShellNoCC {
    packages = with pkgs; [
      # rust
      rustc
      cargo
      rustfmt
      clippy
      rust-analyzer

      # nix
      npins
      nixfmt-tree
      nil
      nixfmt-rfc-style
    ];

    env = {
      # Certain Rust tools won't work without this
      # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
      # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
      RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

      CSS_OPENGRAPH = tailwindOpengraph;
      CSS_LEPTOS_SSG = tailwindLeptosSsg;
    };
  };
}
