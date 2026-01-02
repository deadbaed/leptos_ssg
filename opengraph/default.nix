{
  sources ? import ../npins,
  pkgs ? import sources.nixpkgs { },
  mkTailwindStylesheet ? import ../tailwind.nix { inherit pkgs; },
  supervisord ? import sources.nix-supervisord { inherit pkgs; },
}:
let
  myGeckodriver =
    pkgs.runCommand "run-geckodriver"
      {
        buildInputs = [
          pkgs.firefox
          pkgs.geckodriver
        ];
      }
      ''
        mkdir -p $out/bin
        cat > $out/bin/geckodriver <<EOF
        #!/bin/sh
        export PATH=\${pkgs.firefox}/bin:\$PATH
        exec geckodriver "\$@"
        EOF
        chmod +x $out/bin/geckodriver
      '';
  supervisordProject = supervisord.mkSupervisor {
    project_name = "opengraph";
    paths = supervisord.mkPaths { };
    programs = [
      {
        name = "geckodriver";
        command = "${myGeckodriver}/bin/geckodriver";
      }
    ];
  };
in
{
  tailwind = "${mkTailwindStylesheet "opengraph" ./src true}/style.css";
  # TODO: linux headless option: run with `xvfb-run`

  runWithGeckodriver = pkgs.writeShellScriptBin "runWithGeckodriver" ''
    set -x
    ${supervisordProject.supervisord-wrapper}/bin/supervisord
    sleep 5
    "$@"
    ${supervisordProject.supervisorctl-wrapper}/bin/supervisorctl shutdown
  '';

  supervisordShellHook = supervisordProject.shellHook;

}
