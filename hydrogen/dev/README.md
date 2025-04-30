# Hydrogen // Configuring the dev env

Before entering the Dev Container, copy `dev/_.env` to `dev/.env` and `dev/_application.yml` to `dev/application.yml`, then fill in the necessary values.

## Troubleshooting

### Lavalink not starting: Permission denied

This error is caused by the Lavalink plugins volume not having the correct permissions, because the volume is created by the root user and the Lavalink container runs as the `lavalink` user. To fix this, run the following command:

```
docker run -it --rm --mount type=volume,src=hydrogen-bot_devcontainer_lavalink-plugins,dst=/data alpine:3.21 chmod go+w /data
```

You can change `hydrogen-bot_devcontainer_lavalink-plugins` to the name of the volume that was created for the Lavalink plugins.