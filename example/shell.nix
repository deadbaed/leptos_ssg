{
  headless ? false,
  sources ? import ../npins,
  pkgs ? import sources.nixpkgs { },
  supervisord ? import sources.nix-supervisord { inherit pkgs; },
  leptos_ssg ? import ../. { inherit sources pkgs headless; },
}:
let
  supervisordProject = supervisord.mkSupervisor {
    project_name = "opengraph";
    paths = supervisord.mkPaths { folder = ".dev-supervisor"; };
    programs = [
      {
        name = "watchexec";
        command = "${pkgs.watchexec}/bin/watchexec -N -w content/ cargo run";
      }
      {
        name = "webserver";
        command = "${pkgs.python3}/bin/python -m http.server --directory ./target/example-site 4343";
      }
    ];
  };

  buildWithOpengraph = pkgs.writeShellScriptBin "buildWithOpengraph" ''
    set -x
    opengraph_css=$(mktemp)
    ${leptos_ssg.tailwind.copyTailwindOpengraph}/bin/cp-tailwind-opengraph $opengraph_css
    ${leptos_ssg.opengraph.runWithGeckodriver}/bin/runWithGeckodriver cargo run --release --features opengraph,optimize $opengraph_css
    ${leptos_ssg.tailwind.copyTailwindLeptosSsg}/bin/cp-tailwind-leptos_ssg target/example-site/www/style.css
    rm $opengraph_css
  '';
in

pkgs.mkShellNoCC {
  inherit (leptos_ssg) env;

  packages = leptos_ssg.rustTools ++ [
    supervisordProject.supervisord-wrapper
    supervisordProject.supervisorctl-wrapper
    supervisordProject.supervisord-kill
    pkgs.lnav
    buildWithOpengraph
  ];

  shellHook = ''
    ${supervisordProject.shellHook}
    ${leptos_ssg.opengraph.supervisordShellHook}
  '';
}
