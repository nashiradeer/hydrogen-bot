# Nashira Deer // Hydrogen

Discord music bot powered by Lavalink and focusing on performance, features, usability and practicality.

[![Discord](https://img.shields.io/badge/Discord%20Bot-5865F2?style=for-the-badge&logo=discord&logoColor=%23fff)](https://discord.com/api/oauth2/authorize?client_id=1128087591179268116&permissions=275417975808&scope=bot+applications.commands)

## Building

To build Hydrogen, you need to have [Docker](https://docker.com) (Podman not supported) installed and running on your machine and run `docker build -f Dockerfile.alpine -t hydrogen:latest .` in a terminal with [Docker](https://docker.com) installed and running, after the build is completed, you will have a Docker image ready for use, named "hydrogen:latest".

## Configuring

You can configure Hydrogen using environment variables, here is a list of the available variables:

- DISCORD_TOKEN: Sets the token that will be used to access Discord. (required)
- LAVALINK_HOST: Sets the Lavalink host. (required, e.g. "localhost:2333")
- LAVALINK_PASSWORD: Sets the Lavalink password. (required)
- LAVALINK_TLS: Sets if the Lavalink connection should use TLS. (optional, default: false)

## Credits

Copyright © 2024 Nashira Deer. All rights reserved.
