# Hagaki v1.4-custom

> This version was modified for Rorona Discord Bot - it has colors baked in into frames ahead of time, no longer supports `kindled` frame variants & it now auto loads frame files from disk instead relying on config file. Those changes made average render time 23% faster.

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
  server_name hagaki.rorona-game.com;

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
- **Limited** Customization of said cards (frame/color)
- Ability to save render results into disk and use it later for faster access
- High efficiency
  * Renders runs in parallel
  * Service caches common assets (like pieces of frames) to reduce disk I/O operations
  * Each render is fully reproductible by using same uri, so you can cache http results to reduce number of redundant renders

### Planned features
- None