{
  pkgs,
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
in
mkTailwindStylesheet
