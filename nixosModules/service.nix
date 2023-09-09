{
  config,
  pkgs,
  lib,
  ...
}: let
  gpsRecorder = pkgs.callPackage ./default.nix {};
in {
  options = {
    services.gps-recorder = {
      enable = lib.mkEnableOption "Whether to enable the gps recording service.";

      output-folder = lib.mkOption {
        type = lib.types.str;
        default = "/output/gps";
        description = "The folder to save recordings to.";
      };

      interval-secs = lib.mkOption {
        type = lib.types.int;
        default = 600;
        description = "The interval in seconds in which GPS is recorded.";
      };

      gps-search-duration-ms = lib.mkOption {
        type = lib.types.int;
        default = 1000;
        description = "Time in milliseconds that is spent listening for GPS data for one observation.";
      };
    };
  };

  config = lib.mkIf config.services.gps-recorder.enable {
    services.gpsd = {
      enable = true;
      devices = ["/dev/ttyUSB0"];
    };
    systemd.services.gps-recorder = {
      description = "GPS Recording Service";
      wantedBy = ["multi-user.target"];
      script = ''
        ${pkgs.coreutils}/bin/mkdir -p ${config.services.gps-recorder.output-folder}
        ${gpsRecorder}/bin/gps-recorder --output ${config.services.gps-recorder.output-folder} --interval ${toString config.services.gps-recorder.interval-secs}
      '';
      serviceConfig = {
        User = "root"; # Replace with appropriate user
        Restart = "always";
      };
      startLimitIntervalSec = 0;
    };
  };
}
