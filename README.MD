# Hagaki (Kikuri Card Game's custom asset renderer)

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
  server_name hagaki.kikuri-bot.xyz;

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
`[ GET ]` /render/card/{hash64}
`[ GET ]` /render/fan/{hash64}

### List of features
- Rendering selected card
- Rendering with selected frame
- Rendering dyed frame
- Rendering frame with static decorations
- Caching frames to increase performance by decreasing disk I/O operations
- Rendering card fan
- Paralellized card generation
- Saving rendered card/fan to drive
- Serving saved card and fan from drive

### Planned features
- None