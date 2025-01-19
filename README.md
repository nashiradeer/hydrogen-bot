# Nashira Deer // Hydrogen

Discord music bot powered by Lavalink and focusing on performance, features, usability and practicality.

[![Discord](https://img.shields.io/badge/Discord%20Bot-5865F2?style=for-the-badge&logo=discord&logoColor=%23fff)](https://discord.com/api/oauth2/authorize?client_id=1128087591179268116&permissions=275417975808&scope=bot+applications.commands)

## Developing

To develop Hydrogen, you need to have [Docker](https://docker.com) and [Visual Studio Code](https://code.visualstudio.com) installed or any IDE that supports [Dev Containers](https://containers.dev). But before you enter the Dev Container, please first read the [development guide](dev/README.md) to understand how to configure the development environment.

## Using

Hydrogen is available on our [GitHub Container Registry](https://github.com/nashiradeer/hydrogen-bot/pkgs/container/hydrogen-bot), you can use it by running `docker run -e DISCORD_TOKEN=your_token -e LAVALINK=your_lavalink_host ghcr.io/nashiradeer/hydrogen-bot:0.0.1-alpha.11-alpine` in a terminal with [Docker](https://docker.com) installed and running, replacing `your_token` with your Discord bot token and `your_lavalink_host` with your Lavalink host. You can also use the Debian variant by replacing `ghcr.io/nashiradeer/hydrogen-bot:0.0.1-alpha.11-alpine` with `ghcr.io/nashiradeer/hydrogen-bot:0.0.1-alpha.11-debian`.

## Building

To build Hydrogen, you need to have [Docker](https://docker.com) (Podman not supported) installed and running on your machine and run `docker build -f Dockerfile.alpine -t hydrogen:latest .` in a terminal with [Docker](https://docker.com) installed and running, after the build is completed, you will have a Docker image ready for use, named "hydrogen:latest".

For the Debian variant, you can use `docker build -f Dockerfile.debian -t hydrogen:latest .` instead.

## Configuring

You can configure Hydrogen using environment variables, here is a list of the available variables:

- DISCORD_TOKEN: Sets the token that will be used to access Discord. (required)
- LAVALINK: Sets the Lavalink hosts. (required, e.g. "localhost:2333@youshallnotpass" or "localhost:2333@youshallnotpasslavalink:443@securepassword/tls")

## Credits

Copyright © 2024 Nashira Deer. All rights reserved.
