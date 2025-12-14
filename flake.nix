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
        pkgsStatic = pkgs.pkgsStatic;
        
        # Build static binary using stdenvStatic
        godon-cli = { version ? (if builtins.getEnv "GODON_VERSION" == "" then "DEV_BUILD" else builtins.getEnv "GODON_VERSION") }: pkgs.stdenv.mkDerivation {
          pname = "godon-cli";
          inherit version;
          src = ./.;
          
          nativeBuildInputs = with pkgs; [
            cacert
            nim2
            nimble
            git
            pkg-config
          ];
          
          buildInputs = with pkgsStatic; [
            openssl
            zlib
          ];
          
          hardeningDisable = [ "pie" ];
          
          env = {
            SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            NIX_SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            CURL_CA_BUNDLE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            NIX_CFLAGS_LINK = "-static";
            NIX_LDFLAGS = "-static";
          };
          
          configurePhase = ''
            export HOME=$TMPDIR
          '';
          
          buildPhase = ''
            echo "Building godon-cli version: ${version} (static binary)"
            
            # Refresh package list and install dependencies only
            nimble refresh --verbose
            # Install yaml dependency without building our package
            nimble install -y --depsOnly --verbose
            
            # Build the CLI with static libraries and explicit linking
            mkdir -p bin
            nim c --hints:on --path:src -d:release -d:ssl -d:VERSION="${version}" \
              --passL:"-static" \
              -o:bin/godon_cli src/godon_cli.nim || {
              echo "Compilation failed"
              exit 1
            }
            
            echo "Static build completed successfully!"
            
            # Verify the binary is statically linked
            echo "=== Binary information ==="
            file bin/godon_cli
            echo "=== Dynamic libraries check (should show 'not a dynamic executable' or similar) ==="
            ldd bin/godon_cli || echo "Binary appears to be statically linked (ldd failed as expected)"
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
            platforms = platforms.linux;
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