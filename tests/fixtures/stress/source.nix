{ lib, pkgs, ... }:

let
  mkService = name: port: {
    enable = true;
    settings = {
      inherit port;
      hooks = map (hook: "${name}-${hook}") [ "before" "after" ];
    };
  };
in
{
  services.web = mkService "web" 8080;
  packages = lib.mapAttrs (_name: value: value.package) {
    api = { package = pkgs.curl; };
    ui = { package = pkgs.nodejs; };
  };
}
