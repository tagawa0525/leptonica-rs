{
  description = "leptonica-rs development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # Rust ツールチェーンを flake.lock で固定するために使用する。
    # nixpkgs の rustc と違い、rust-src / rust-analyzer などの
    # コンポーネントやターゲットを宣言的に指定できる。
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      pkgsFor = forAllSystems (
        system:
        import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        }
      );
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor.${system};

          # default プロファイルに rustc / cargo / rustfmt / clippy が含まれる。
          # rust-src と rust-analyzer は rust-analyzer の補完・定義ジャンプに必要。
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            packages = [
              rustToolchain
              # .config/nextest.toml のプロファイル（slow-timeout、JUnit 出力）を
              # ローカルで使うため同梱する。CI は cargo test を使用。
              pkgs.cargo-nextest
            ];
          };
        }
      );
    };
}
