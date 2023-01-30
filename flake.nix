{
  description =
    "Forward Alertmanager alerts to gotify push notification server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nmattia/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, naersk }:
    let
      cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" ];
      forAllSystems = f:
        nixpkgs.lib.genAttrs supportedSystems (system: f system);
    in
    {
      overlay = final: prev: {
        "${cargoToml.package.name}" = final.callPackage ./. { inherit naersk; };
      };

      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ self.overlay ];
          };
        in
        { "${cargoToml.package.name}" = pkgs."${cargoToml.package.name}"; });

      nixosModules."alertmanager-forwarder" = { config, lib, pkgs, ... }:
        let
          cfg = config.services.alertmanager-forwarder;
          settingsFormat = pkgs.formats.toml { };
        in
        {
          options = {
            services.alertmanager-forwarder = {
              enable = lib.mkEnableOption
                "Forwards alerts from alertmanager to services.";
              settings = lib.mkOption {
                type = settingsFormat.type;
                default = { };
              };
            };
          };

          config = lib.mkIf cfg.enable {
            systemd.services.alertmanager-forwarder = {
              wantedBy = [ "multi-user.target" ];
              after = [ "network.target" ];
              path = [ self.packages."x86_64-linux".alertmanager-forwarder ];
              script = ''
                ${
                  self.packages."x86_64-linux".alertmanager-forwarder
                }/bin/alertmanager-forwarder --config \
                  ${settingsFormat.generate "config.toml" cfg.settings} \
              '';
              serviceConfig = {
                Type = "simple";
                Environment = "RUST_LOG=info";
                ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
                Restart = "on-failure";
                StateDirectory = "alertmanager-forwarder";
              };
            };
          };
        };

      defaultPackage = forAllSystems (system:
        (import nixpkgs {
          inherit system;
          overlays = [ self.overlay ];
        })."${cargoToml.package.name}");

      checks = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ self.overlay ];
          };
        in
        {
          format = pkgs.runCommand "check-format"
            {
              buildInputs = with pkgs; [ rustfmt cargo ];
            } ''
            ${pkgs.rustfmt}/bin/cargo-fmt fmt --manifest-path ${
              ./.
            }/Cargo.toml -- --check
            ${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt --check ${./.}
            touch $out # it worked!
          '';
          "${cargoToml.package.name}" = pkgs."${cargoToml.package.name}";
        });
      devShell = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ self.overlay ];
          };
        in
        pkgs.mkShell {
          inputsFrom = with pkgs; [ pkgs."${cargoToml.package.name}" ];
          buildInputs = with pkgs; [ rustfmt nixpkgs-fmt ];
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        });
    };
}
