Although it's named `CONTRIBUTING`, this is a general document on
useful things I (Caer) learned while building out Cosi. In no particular order.

## Getting Perspective: `isometric` + `dimetric` Projections

In the context of 2D video game graphics, the word **Isometric** conjures
up imagery of "2.5D" games--those beautiful, two-dimensional games that
have the illusion of being partially or completely three-dimensional.

When I began coding Cosi, I _knew_ I wanted the graphics to render in a 2D
space with 2.5D graphics. 3D spaces and graphics _are_ the "standard" for
simulating systems [like robot swarms](http://argos-sim.info), but in many
cases only two of the three dimensions get used.

Using some isometric tiles and projection algorithms from the internet,
I was able to (fairly) easily set up a renderer for isometric tiles. However,
when I went to make _my own_ tiles in isometric space, I noticed that the
rendered tiles looked a little _tall_, and had lots of empty space between them.

Isometric Tiles | Dimetric Tiles (2:1)
-|-
![An Isometric Grid of Isometric Tiles](/docs/images/isometric-grid.png) | ![An Isometric Grid of Dimetric Tiles](/docs/images/dimetric-grid.png)

> Table: Closeup images of grids drawn using the same isometric coordinate system, but with different tile geometries.

What I _didn't_ realize was that most "isometric" game graphics are actually
built using **Dimetric** projections, _not_ true isometric projections:

- Isometric projections represent the three faces of an object with _equal_ proportions.
  These proportions are defined by 120° angles between each face.
- Dimetric projections represent two faces of an object with equal proportion,
  and a third face with different proportions.

The most common dimetric projection in video game art is the "**2:1 isometric**"
projection, which defines the two lower faces of a given object by 105° angles, 
and the upper third face by a 150° angle. In this projection, the upper third
face is _exactly_ twice as wide as it is tall.

## TODO: Pixels and Texels

```rust
/// Default resolution in **Texels** ("Texture Pixels")
/// of a single tile.
const DEFAULT_TILE_TEXELS: f32 = 16.0;

/// Approximately 320x180 texels when the
/// tile size is 16 texels.
const DEFAULT_CANVAS_TILES: (usize, usize) = (20, 11);

/// Number of logical grid units per tile.
const DEFAULT_UNITS_PER_TILE: f32 = 1.0;

/// Canvas resolution.
const DEFAULT_CANVAS_TEXELS: (f32, f32) = (320.0, 180.0);
```