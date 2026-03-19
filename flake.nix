{
  description = "Godon CLI - Rust-based CLI for Godon API";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default;

        godon-cli = { version ? (if builtins.getEnv "GODON_VERSION" == "" then
          "DEV_BUILD"
        else
          builtins.getEnv "GODON_VERSION") }:
          pkgs.stdenv.mkDerivation {
            pname = "godon-cli";
            inherit version;
            src = ./.;

            nativeBuildInputs = with pkgs; [ rustToolchain pkg-config ];

            buildInputs = with pkgs; [ openssl ];

            env = {
              CARGO_HOME = "/build/cargo-home";
              SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
              NIX_SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
              CURL_CA_BUNDLE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            };

            buildPhase = ''
              echo "Building godon-cli version: ${version}"
              mkdir -p $CARGO_HOME
              cargo build --release
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp target/release/godon_cli $out/bin/godon_cli
              chmod +x $out/bin/godon_cli
            '';

            meta = with pkgs.lib; {
              description = "CLI for the Godon API";
              license = licenses.agpl3Only;
              platforms = platforms.linux;
            };
          };

      in {
        packages.default = godon-cli { };
        packages.godon-cli = godon-cli;

        packages.godon-cli-custom = version: godon-cli { inherit version; };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [ rustToolchain rust-analyzer cargo-watch ];

          shellHook = ''
            echo "Godon CLI development environment"
            echo "Rust: $(rustc --version)"
          '';
        };
      });
}
