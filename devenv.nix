{ pkgs, config, ... }:

{
  packages = [
    pkgs.pkg-config
    pkgs.openssl
  ];

  languages.rust.enable = true;

  languages.javascript = {
    enable = true;
    directory = "./dashboard/frontend";
    bun = {
      enable = true;
      install.enable = true;
    };
  };

  scripts.run.exec = ''
    set -euo pipefail
    (cd "${config.devenv.root}/dashboard/frontend" && bun run build)
    cd "${config.devenv.root}/dashboard"
    exec cargo run
  '';

  scripts.run-mock.exec = ''
    set -euo pipefail
    export MOCK_DB=1
    (cd "${config.devenv.root}/dashboard/frontend" && bun run build)
    cd "${config.devenv.root}/dashboard"
    exec cargo run
  '';
}
