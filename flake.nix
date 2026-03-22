{
  description = "A basic tool to handle OCI images without root or daemons.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
    inputs = with pkgs; [
      cargo
      rustc
      rustfmt
      clippy
      pkg-config
    ];
  in {
    devShells."x86_64-linux".default = pkgs.mkShell {
      buildInputs = inputs;
      nativeBuildInputs = [pkgs.pkg-config];
      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    };
  };
}
