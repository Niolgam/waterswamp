{
  description = "Ambiente Rust para Waterswamp";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        # Ferramentas que rodam no momento de build (como compiladores e pkg-config)
        nativeBuildInputs = [
          pkgs.pkg-config 
          # Se estiver usando rust via overlay, mantenha a linha abaixo,
          # senão o nix vai usar o cargo padrão do sistema se você rodar `nix develop`
          pkgs.rust-bin.stable.latest.default 
        ];

        # Bibliotecas que seu código precisa linkar (OpenSSL)
        buildInputs = [
          pkgs.openssl
        ];

        # Variáveis de ambiente úteis para garantir que o openssl-sys ache tudo
        shellHook = ''
          export OPENSSL_DIR="${pkgs.openssl.dev}"
          export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
          export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
        '';
      };
    };
}
