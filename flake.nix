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
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rust-custom-toolchain = (pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        });

        craneLib =
          (inputs.crane.mkLib pkgs).overrideToolchain rust-custom-toolchain;

      in {
        packages.default = pkgs.lib.makeOverridable ({

          # the location of the flake lock to get a bearing on
          flakelock
          # how old the flake can be before it is out of date
          , threshold
          # the i3status icon the bar will be displayed with
          , icon, ... }:
          with pkgs;
          let
            # read in flake.lock from location
            # iterate through and find the most recent modified date in all inputs
            # the user is expected to not update only specific entries in the flake so
            # we can just take the most recent thing as an indication of when the flake was last updated
            # and we're going to ignore that sometimes flakes just don't receive updates because nixpkgs is being constantly updated
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

            src = ./.;

            prePatch = ''
              cp ${config_file} ./src/modified_data.rs
            '';

            cargoArtifacts = craneLib.buildDepsOnly { inherit src; };
          in craneLib.buildPackage { inherit cargoArtifacts src prePatch; }) {
            flakelock = ./flake.lock;
            threshold = 14;
            icon = "cogs";
          };

        checks = let
          src = ./.;

          cargoArtifacts = craneLib.buildDepsOnly { inherit src; };
          build-tests = craneLib.buildPackage { inherit cargoArtifacts src; };
        in {
          inherit build-tests;

          # Run clippy (and deny all warnings) on the crate source,
          # again, resuing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          my-crate-clippy = craneLib.cargoClippy {
            inherit cargoArtifacts src;
            cargoClippyExtraArgs = "-- --deny warnings";
          };

          # Check formatting
          my-crate-fmt = craneLib.cargoFmt { inherit src; };

          # Run tests with cargo-nextest
          my-crate-nextest = craneLib.cargoNextest {
            inherit cargoArtifacts src;
            partitions = 1;
            partitionType = "count";
          };
        };

      });
}
