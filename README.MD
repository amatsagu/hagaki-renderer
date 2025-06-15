# Hagaki v1.2 (custom asset renderer for Yuli Discord Bot)

### Building
To build release binary use command
```sh
make build
```

### How to release new version on final server
1. Clone all files to `/home/ubuntu/hagaki`
2. Build app: `make build`
3. Reload service: `sudo make reload-service`

### Nginx config
```nginx
server {
  listen 80;
  server_name hagaki.yuli-bot.com;

  # Security headers
  add_header X-Content-Type-Options nosniff;
  add_header X-Frame-Options SAMEORIGIN;
  add_header X-XSS-Protection "1; mode=block";

  location / {
    # Hagaki service is expected to use 8899 port
    proxy_pass http://127.0.0.1:8899;
  }
}
```

### API Routes
`[ GET ]` /render/album/{hash64} <br>
`[ GET ]` /render/card/{hash64} <br>
`[ GET ]` /render/fan/{hash64}

### List of features
- Ability to request render of single card, fan or album of cards
- Customization of said cards (frame, color, kindled state)
- Ability to save render results into disk and use it later for faster access
- High efficiency
  * Renders runs in parallel
  * Service caches common assets (like pieces of frames) to reduce disk I/O operations
  * Each render can be recreated 1:1 by using same uri, so you can cache http results to reduce number of redundant renders

### Planned features
- None