{
  sources ? import ../npins,
  pkgs ? import sources.nixpkgs { },
  lib ? pkgs.lib,
  mkTailwindStylesheet ? import ../tailwind.nix { inherit pkgs; },
  supervisord ? import sources.nix-supervisord { inherit pkgs; },
  headless ? false,
}:
let
  myGeckodriver =
    pkgs.runCommand "run-geckodriver"
      {
      }
      ''
        mkdir -p $out/bin
        cat > $out/bin/geckodriver <<EOF
        #!/bin/sh
        export PATH=\${pkgs.firefox}/bin:\$PATH
        exec ${lib.optionalString (headless && pkgs.stdenv.isLinux) "${pkgs.xvfb-run}/bin/xvfb-run --auto-servernum "}${pkgs.geckodriver}/bin/geckodriver "\$@"
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
  runWithGeckodriver = pkgs.writeShellScriptBin "runWithGeckodriver" ''
    set -x
    ${supervisordProject.supervisord-wrapper}/bin/supervisord
    sleep 5
    "$@"
    exit_code=$?
    ${supervisordProject.supervisorctl-wrapper}/bin/supervisorctl shutdown
    exit $exit_code
  '';

  supervisordShellHook = supervisordProject.shellHook;

}
