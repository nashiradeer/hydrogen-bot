# Nashira Deer // Hydrogen

Discord music bot powered by Lavalink and focusing on performance, features, usability and practicality.

[![Discord](https://img.shields.io/badge/Discord%20Bot-5865F2?style=for-the-badge&logo=discord&logoColor=%23fff)](https://discord.com/api/oauth2/authorize?client_id=1128087591179268116&permissions=275417975808&scope=bot+applications.commands)

## Donating

You can help Hydrogen's development by donating.

[![GitHub Sponsor](https://img.shields.io/badge/GitHub%20Sponsor-181717?style=for-the-badge&logo=github&logoColor=%23fff)
](https://github.com/sponsors/nashiradeer)
[![Patreon](https://img.shields.io/badge/Patreon-%23000?style=for-the-badge&logo=patreon&logoColor=%23fff)
](https://www.patreon.com/nashiradeer)
[![Pix](https://img.shields.io/badge/Pix-%2377B6A8?style=for-the-badge&logo=pix&logoColor=%23fff)](https://pixie.gg/nashiradeer)

## Developing

To develop Hydrogen, you need to have [Docker](https://docker.com)
and [Visual Studio Code](https://code.visualstudio.com) installed or any IDE that
supports [Dev Containers](https://containers.dev). But before you enter the Dev Container, please first read
the [development guide](dev/README.md) to understand how to configure the development environment.

If you are using JetBrains IDEs, unfortunately, Dev Containers support is very limited, so it is recommended to develop
on the host machine instead using Rust 1.85.0.

## Using

Hydrogen is available on
our [GitHub Container Registry](https://github.com/nashiradeer/hydrogen-bot/pkgs/container/hydrogen-bot), you can use it
by running
`docker run -e DISCORD_TOKEN=your_token -e LAVALINK=your_lavalink_host ghcr.io/nashiradeer/hydrogen-bot:0.0.1-alpha.14-alpine`
in a terminal with [Docker](https://docker.com) installed and running, replacing `your_token` with your Discord bot
token and `your_lavalink_host` with your Lavalink host. You can also use the Debian variant by replacing
`ghcr.io/nashiradeer/hydrogen-bot:0.0.1-alpha.14-alpine` with `ghcr.io/nashiradeer/hydrogen-bot:0.0.1-alpha.14-debian`.

## Building

To build Hydrogen, you need to have [Docker](https://docker.com) (Podman not supported) installed and running on your
machine and run `docker build -f Dockerfile.alpine -t hydrogen:latest .` in a terminal with [Docker](https://docker.com)
installed and running, after the build is completed, you will have a Docker image ready for use, named "hydrogen:
latest".

For the Debian variant, you can use `docker build -f Dockerfile.debian -t hydrogen:latest .` instead.

Hydrogen by default uses SIMD instructions to parse JSON, to disable it, you need to build the Hydrogen without the
default features or by removing the `simd-json` feature from the default features in the `Cargo.toml` file.

## Configuring

You can configure Hydrogen using environment variables, here is a list of the available variables:

- DISCORD_TOKEN: Sets the token that will be used to access Discord. (required)
- LAVALINK: Sets the Lavalink hosts. (required, e.g. `localhost:2333@youshallnotpass` or
  `localhost:2333@youshallnotpass;lavalink:443@securepassword/tls`)
- DISABLE_MULTI_THREADING: Disables multi-threading. (optional, default: false)

## License

Created by [Nashira Deer](https://www.nashiradeer.com) and licensed under [General Public License v3.0](https://github.com/nashiradeer/hydrogen-bot/blob/main/LICENSE.txt).
