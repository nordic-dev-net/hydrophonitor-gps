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

      package = lib.mkOption {
        type = lib.types.str;
        default = gpsRecorder;
        description = "The package to use for the gps recorder.";
      };

      output-path = lib.mkOption {
        type = lib.types.str;
        default = "/output/gps";
        description = "The folder to save recordings to.";
      };

      interval-secs = lib.mkOption {
        type = lib.types.int;
        default = 600;
        description = "The interval in seconds in which GPS is recorded.";
      };

      devices = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = ["/dev/ttyUSB0"];
        description = "The devices to use for GPS recording.";
      };

      user = lib.mkOption {
        type = lib.types.str;
        default = "root";
        description = "The user to run the gps recorder as.";
      };

      hostname = lib.mkOption {
        type = lib.types.str;
        default = "localhost";
        description = "The hostname to use for the gps recorder.";
      };

      port = lib.mkOption {
        type = lib.types.int;
        default = 2947;
        description = "The port to use for the gps recorder.";
      };
    };
  };

  config = lib.mkIf config.services.gps-recorder.enable {
    services.gpsd = {
      enable = true;
      devices = config.services.gps-recorder.devices;
    };
    systemd.services.gps-recorder = {
      description = "GPS Recording Service";
      wantedBy = ["multi-user.target"];

      script = ''
        #!/usr/bin/env bash
        set -x
        ${pkgs.coreutils}/bin/mkdir -p ${config.services.gps-recorder.output-path}
        ${gpsRecorder}/bin/gps-recorder \
        --output-path ${config.services.gps-recorder.output-path} \
        --interval ${toString config.services.gps-recorder.interval-secs} \
        --hostname ${config.services.gps-recorder.hostname} \
        --port ${toString config.services.gps-recorder.port} \
      '';

      serviceConfig = {
        User = config.services.gps-recorder.user;
        Restart = "always";
      };
    };
  };
}
