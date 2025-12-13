{
  description = "Godon CLI - Nim-based CLI for Godon API";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        
        # Build using nimble following the godon-api pattern
        godon-cli = { version ? builtins.getEnv "GODON_VERSION" or "DEV_BUILD" }: pkgs.stdenv.mkDerivation {
          pname = "godon-cli";
          inherit version;
          src = ./.;
          
          nativeBuildInputs = with pkgs; [
            cacert
            nim2
            nimble
            git
            openssl.dev
          ];
          
          env = {
            SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            NIX_SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            CURL_CA_BUNDLE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          };
          
          configurePhase = ''
            export HOME=$TMPDIR
          '';
          
          buildPhase = ''
            echo "Building godon-cli version: ${version}"
            
            # Refresh package list and install dependencies only
            nimble refresh --verbose
            # Install yaml dependency without building our package
            nimble install -y --depsOnly --verbose
            
            # Build the CLI
            mkdir -p bin
            nim c --hints:on --path:src -d:release -d:VERSION="${version}" -o:bin/godon_cli src/godon_cli.nim || {
              echo "Compilation failed"
              exit 1
            }
            
            echo "Build completed successfully!"
          '';
          
          installPhase = ''
            mkdir -p $out/bin
            
            # Install the binary
            cp bin/godon_cli $out/bin/godon_cli
            chmod +x $out/bin/godon_cli
          '';
          
          meta = with pkgs.lib; {
            description = "CLI for the Godon API";
            license = licenses.agpl3Only;
            platforms = platforms.all;
          };
        };
        
      in {
        packages.default = godon-cli { };
        packages.godon-cli = godon-cli;
        
        # Allow building with custom version
        packages.godon-cli-custom = version: godon-cli { inherit version; };
        
        # Development shell with Nim and build tools
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            nim2
            nimble
            git
          ];
          
          shellHook = ''
            echo "Godon CLI development environment"
            echo "Nim: $(nim --version | head -n1)"
            echo "Nimble: $(nimble --version | head -n1)"
          '';
        };
      });
}