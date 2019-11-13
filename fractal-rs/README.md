Coordinate Systems:
* Tile Space: x index, y index, zoom level
* Viewport Space: complex space center, zoom level
* Complex Space: top left, bottom right
* Sample Space: tile space + pixel offset x, y

Phases:
1. Compose - Derive tile information
2. Generate - Perform algorithm on tiles to generate fractal samples
3. Render - Sample tiles and color onto the screen

* Viewport Config: everything needed to be choosen to complete a render
* Viewport Info: everything possible to derive from config prior to generation

Sampling:
1. Have a complex space coordinate and zoom level
2. Snap allowed error to a location in tile space
 * Allowed error should be within the square of the pixel

Render phases:

1. Define ViewportConfig
	* Window Dimensions
	* Zoom
	* Algorithm
		* Iterations
	* Colorizer
		* Palette
	* Allowed error?
2. Derive ViewportInfo
	* Complex space coords
	* TileInfo list - index, key, etc
	* Step Size
	* Tile Scale
4. Generate Tiles - generator
	* Fall through to cache?
5. Render tiles
	1. Choose tile
	2. Map position viewport space to tile + sample space
	3. Supersample? (avg of region)
	4. Colorize
