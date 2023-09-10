{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.gpsd
  ];

  shellHook = ''
  gpsd -D 5 -N -n /dev/ttyUSB0
  '';
}
