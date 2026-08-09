{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [
                rust-overlay.overlays.default
                self.overlays.default
              ];
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          let
            rust = prev.rust-bin;
          in
          if builtins.pathExists ./rust-toolchain.toml then
            rust.fromRustupToolchainFile ./rust-toolchain.toml
          else if builtins.pathExists ./rust-toolchain then
            rust.fromRustupToolchainFile ./rust-toolchain
          else
            rust.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rustfmt"
              ];
            };
      };

      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustToolchain
              openssl
              pkg-config
              rust-analyzer
              cargo-deny
              cargo-edit
              cargo-watch
              bacon

              # App Dependencies
              pacman
              llvmPackages.libclang
              libarchive

              fish
            ];

            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
              LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
            };

            shellHook = ''
              echo "Enabling dev environment"
              echo "`${pkgs.rustToolchain}/bin/rustc --version`"
              # From: https://github.com/NixOS/nixpkgs/blob/1fab95f5190d087e66a3502481e34e15d62090aa/pkgs/applications/networking/browsers/firefox/common.nix#L247-L253
              # Set C flags for Rust's bindgen program. Unlike ordinary C
              # compilation, bindgen does not invoke $CC directly. Instead it
              # uses LLVM's libclang. To make sure all necessary flags are
              # included we need to look in a few places.
              export BINDGEN_EXTRA_CLANG_ARGS="$(< ${pkgs.stdenv.cc}/nix-support/libc-crt1-cflags) \
                $(< ${pkgs.stdenv.cc}/nix-support/libc-cflags) \
                $(< ${pkgs.stdenv.cc}/nix-support/cc-cflags) \
                $(< ${pkgs.stdenv.cc}/nix-support/libcxx-cxxflags) \
                -idirafter ${pkgs.libarchive.dev}/include\
                ${pkgs.lib.optionalString pkgs.stdenv.cc.isClang "-idirafter ${pkgs.stdenv.cc.cc}/pkgs.lib/clang/${pkgs.lib.getVersion pkgs.stdenv.cc.cc}/include"} \
                ${pkgs.lib.optionalString pkgs.stdenv.cc.isGNU "-isystem ${pkgs.stdenv.cc.cc}/include/c++/${pkgs.lib.getVersion pkgs.stdenv.cc.cc} -isystem ${pkgs.stdenv.cc.cc}/include/c++/${pkgs.lib.getVersion pkgs.stdenv.cc.cc}/${pkgs.stdenv.hostPlatform.config} -idirafter ${pkgs.stdenv.cc.cc}/pkgs.lib/gcc/${pkgs.stdenv.hostPlatform.config}/${pkgs.lib.getVersion pkgs.stdenv.cc.cc}/include"} \
              "
              exec fish
            '';
          };
        }
      );
    };
}
