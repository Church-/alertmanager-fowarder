{ pkgs, stdenv, lib, rustPlatform, }:

rustPlatform.buildRustPackage rec {
  pname = "alertmanager-forwarder";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  buildInputs = with pkgs; [ pkgconfig openssl ];
  nativeBuildInputs = with pkgs; [
    pkgconfig
    openssl
    makeWrapper
  ];

  postInstall = ''
    wrapProgram "$out/bin/alertmanager_forwarder" \
      --prefix ROCKET_PORT : "6064" \
      --prefix ROCKET_ADDRESS : "127.0.0.1"
  '';
  
}
