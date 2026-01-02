{
  sources ? import ./npins,
  pkgs ? import sources.nixpkgs { },
  mkTailwindStylesheet ? import ./tailwind.nix { inherit pkgs; },
  headless ? false,
  opengraph ? import ./opengraph { inherit sources pkgs mkTailwindStylesheet headless; },
}:

let
  copyTailwindLeptosSsg = pkgs.writeShellScriptBin "cp-tailwind-leptos_ssg" ''
    cp ${mkTailwindStylesheet "leptos_ssg" ./src true}/style.css "$@"
  '';
  copyTailwindOpengraph = pkgs.writeShellScriptBin "cp-tailwind-opengraph" ''
    cp ${opengraph.tailwind} "$@"
  '';
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

      # node
      nodejs

      # nix
      npins
      nixfmt-tree
      nil
      nixfmt-rfc-style

      # tailwind stylesheets
      copyTailwindLeptosSsg
      copyTailwindOpengraph

      # opengraph
      opengraph.runWithGeckodriver
    ];

    env = {
      # Certain Rust tools won't work without this
      # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
      # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
      RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    };

    shellHook = opengraph.supervisordShellHook;
  };
}
