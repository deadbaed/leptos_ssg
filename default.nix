{
  sources ? import ./npins,
  pkgs ? import sources.nixpkgs { },
  mkTailwindStylesheet ? import ./tailwind.nix { inherit pkgs; },
  headless ? false,
  opengraph ? import ./opengraph {
    inherit
      sources
      pkgs
      mkTailwindStylesheet
      headless
      ;
  },
}:

let
  copyTailwindLeptosSsg = pkgs.writeShellScriptBin "cp-tailwind-leptos_ssg" ''
    cp ${mkTailwindStylesheet "leptos_ssg" ./src true}/style.css "$@"
  '';
  copyTailwindOpengraph = pkgs.writeShellScriptBin "cp-tailwind-opengraph" ''
    cp ${opengraph.tailwind} "$@"
  '';
  tailwind = {
    inherit copyTailwindLeptosSsg copyTailwindOpengraph;
  };
  # TODO: nix module to build the crate, with different options (opengraph, optimize, release)
  # and export it for consumers
  # instead of calling raw `cargo build` commands
  rustTools = with pkgs; [
    rustc
    cargo
    rustfmt
    clippy
    rust-analyzer
  ];
  env = {
    # Certain Rust tools won't work without this
    # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
    # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  };
in
{
  inherit
    rustTools
    opengraph
    tailwind
    env
    ;

  shell = pkgs.mkShellNoCC {
    env = env;

    packages =
      with pkgs;
      rustTools
      ++ [
        # node
        nodejs

        # nix
        npins
        nixfmt-tree
        nil
        nixfmt-rfc-style

        # opengraph
        opengraph.runWithGeckodriver

        tailwind.copyTailwindLeptosSsg
        tailwind.copyTailwindOpengraph
      ];

    shellHook = opengraph.supervisordShellHook;
  };
}
