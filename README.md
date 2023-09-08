# Microblog-bot
A discord bot that lets you upload blog posts to your blog

## Deploying using docker compose
Prerequisites:
- Docker CLI
- Docker compose

1. Clone this repository: `git clone https://github.com/abdann/microblog-bot.git`
2. Build the docker image (**IMPORTANT**: If you're building for a different target platform, make sure you add the flag `--platform <PLATFORM>` to target the correct host operating system instead of the build host's platform.): `cd microblog-bot; docker buildx build --tag abdann/microblogbot:latest .`
3. Make a new directory to store the bot's data and configuration files: `cd ..; mkdir microblog-docker; cd microblog-docker`
4. Make a new file called `docker-compose.yml` and copy the following into it:

```
version: '3.8'
services:
  website:
    user: "${USER_ID}:${GROUP_ID}"
    image: abdann/microblogbot:latest
    container_name: microblogbot
    hostname: microblogbot
    restart: unless-stopped
    environment:
      # The bot's discord token
      - DISCORD_TOKEN=<DISCORDTOKEN>
      # The blog login username
      - BLOG_USERNAME=<USERNAME>
      # The blog login password
      - BLOG_PASSWORD=<PASSWORD>
      # The blog host
      - BLOG_HOSTNAME=<HOSTNAME>
      # The blogpost URL endpoint
      - POST_ENDPOINT=<POSTENDPOINT>
      # The login URL endpoint
      - LOGIN_ENDPOINT=<LOGINENDPOINT>
      # The scheme to use (either 'http' or 'https'. This should always be set to 'https' to prevent leaking of credentials, unless the blog website and bot are running on the same host).
      - SCHEME=https
    volumes:
      # Directory to store log files
      - ./logs/:/microblogbot/logs/
```

**IMPORTANT**: Make sure to replace the things with angle brackets with the correct information as necessary for your configuration (ex: `<USERNAME>` becomes `microblog`)

5. Make another file called `.env` and copy the following into it:
```
USER_ID=1000
GROUP_ID=1000
```
6. Make a new directory to store logs: `mkdir logs`
7. Bring up the project: `docker compose up -d`
