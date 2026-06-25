{
  description = "sigi — symbolic esoteric stack language that compiles to C";

  # Use NixOS 25.11 (stable) for a reproducible dev environment.
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";

  outputs = { self, nixpkgs }:
    let
      # Support both x86 and ARM on Linux and macOS.
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in {
      # Dev shell with Rust toolchain and GCC (for the C compilation step and tests).
      devShells = forAllSystems (system:
        let pkgs = nixpkgs.legacyPackages.${system}; in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustc
              cargo
              gcc       # needed to compile generated C from --run
              pkg-config
            ];

            shellHook = ''
              echo "sigi dev shell ready (Rust + GCC)"
            '';
          };
        }
      );

      formatter = forAllSystems (pkgs: pkgs.nixfmt-rfc-style);
    };
}
