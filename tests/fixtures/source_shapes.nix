{ pkgs, lib, ... }:
let
  local = "x";
in {
  services.demo.enable = true;
  packages = [ pkgs.git ];
  nested = { value = local; };
}
