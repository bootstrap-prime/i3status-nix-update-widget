{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    nixpkgs-21.url = "github:NixOS/nixpkgs/nixos-21.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = inputs@{ self, flake-utils, nixpkgs, rust-overlay, nixpkgs-21, crane
    , advisory-db, ... }:
    flake-utils.lib.eachSystem [ flake-utils.lib.system.x86_64-linux ] (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rust-custom-toolchain = (pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        });

        craneLib =
          (inputs.crane.mkLib pkgs).overrideToolchain rust-custom-toolchain;

      in {
        # devShell = pkgs.mkShell {
        #   buildInputs = with pkgs; [ ];

        #   nativeBuildInputs = with pkgs; [
        #     # get current rust toolchain defaults (this includes clippy and rustfmt)
        #     rust-custom-toolchain

        #     cargo-edit
        #   ];

        #   # fetch with cli instead of native
        #   CARGO_NET_GIT_FETCH_WITH_CLI = "true";
        #   RUST_BACKTRACE = 1;
        # };

        packages.default = (self.packages.${system}.defaultthing { });

        packages.defaultthing =
          { flakelock ? ./flake.lock, threshold ? 14, icon ? "cogs", ... }:
          with pkgs;
          let
            # flakelock = ./flake.lock;
            # threshold = 14;
            # icon = "cogs";
            lockfile = builtins.fromJSON (builtins.readFile flakelock);
            recenttime = builtins.head (lib.sort (a: b: a > b)
              (map (key: lockfile.nodes.${key}.locked.lastModified or 0)
                (lib.attrNames lockfile.nodes)));

            config_file = pkgs.writeText "modified_data.rs" ''
              const MODIFIED_DATE: i64 = ${toString recenttime};
              const GOOD_THRESHOLD: i64 = 3;
              const UPDATE_THRESHOLD: i64 = 4;
              const OUT_OF_DATE_THRESHOLD: i64 = ${toString threshold};
              const STATUS_ICON: &str = "${icon}";
            '';

            cargoArtifacts = craneLib.buildDepsOnly {
              src = ./.;
              buildInputs = [ ];
            };
          in craneLib.buildPackage {
            inherit cargoArtifacts;
            src = ./.;
            buildInputs = [ ];
            patchPhase = ''
              cp ${config_file} $src/modified_data.rs
            '';
          };

        # nixosModule = { config, lib, pkgs, ... }:
        #   with lib;
        #   let cfg = config.programs.updatewidget;
        #   in {
        #     options.programs.updatewidget = {
        #       enable = mkEnableOption "Enables the nix flake update widget";

        #       flakelock = mkOption {
        #         type = types.str;
        #         example = "./flake.lock";
        #         description =
        #           "the location of the flake lock to get a bearing on";
        #       };

        #       icon = mkOption {
        #         type = types.str;
        #         default = "cogs";
        #         description = "the i3status icon to place next to the age.";
        #       };

        #       threshold = mkOption {
        #         type = types.int;
        #         example = 3;
        #         description =
        #           "the number of days after which the flake is out of date";
        #       };
        #     };

        #     config = let
        #       # read in flake.lock from location
        #       # iterate through and find the most recent modified date in all inputs
        #       # the user is expected to not update only specific entries in the flake so
        #       # we can just take the most recent thing as an indication of when the flake was last updated
        #       # and we're going to ignore that sometimes flakes just don't receive updates because nixpkgs is being constantly updated

        #     in mkIf cfg.enable {

        #     };
        #   };

        # checks = let
        #   src = ./.;

        #   cargoArtifacts = craneLib.buildDepsOnly {
        #     inherit src;
        #     buildInputs = with pkgs; [ openssl pkg-config ];
        #   };
        #   build-tests = craneLib.buildPackage {
        #     inherit cargoArtifacts src;
        #     buildInputs = with pkgs; [ openssl pkg-config capnproto ];
        #   };
        # in {
        #   inherit build-tests;

        #   # Run clippy (and deny all warnings) on the crate source,
        #   # again, resuing the dependency artifacts from above.
        #   #
        #   # Note that this is done as a separate derivation so that
        #   # we can block the CI if there are issues here, but not
        #   # prevent downstream consumers from building our crate by itself.
        #   my-crate-clippy = craneLib.cargoClippy {
        #     inherit cargoArtifacts src;
        #     cargoClippyExtraArgs = "-- --deny warnings";

        #     buildInputs = with pkgs; [ openssl pkg-config capnproto ];
        #   };

        #   # Check formatting
        #   my-crate-fmt = craneLib.cargoFmt { inherit src; };

        #   # Audit dependencies
        #   my-crate-audit = craneLib.cargoAudit {
        #     inherit src;
        #     advisory-db = inputs.advisory-db;
        #     cargoAuditExtraArgs = "--ignore RUSTSEC-2020-0071";
        #   };

        #   # Run tests with cargo-nextest
        #   my-crate-nextest = craneLib.cargoNextest {
        #     inherit cargoArtifacts src;
        #     partitions = 1;
        #     partitionType = "count";

        #     buildInputs = with pkgs; [ openssl pkg-config capnproto ];
        #   };
        # };

      });
}
