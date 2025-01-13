# Hagaki

## Building
To build release binary use command
```
cargo build --release
```

## Completed features
- Rendering selected card
- Rendering with selected frame
- Rendering dyed frame
- Rendering frame with static decorations
- Caching frames to increase performance by decreasing disk I/O operations
- Rendering card fan (basic version, supports 1, 3, 5, and 7 card fans)
- Paralellized card generation

## TODO
- Rendering card fan - fix card positioning, test with random frames
- Saving rendered card to drive

## SOON™
- Rendering profiles/postcards