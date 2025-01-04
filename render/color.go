package render

import (
	"image"
	"image/color"
)

func ColorValueToRGBA(x uint32) color.RGBA {
	return color.RGBA{uint8((x >> 16) & 0xff), uint8((x >> 8) & 0xff), uint8(x & 0xff), 255}
}

func Recolor(canvas image.Image, tintColor color.RGBA) *image.RGBA {
	x, y := 0, 0
	max := canvas.Bounds().Max
	newCanvas := image.NewRGBA(image.Rectangle{image.Point{}, max})

	for x != max.X && y != max.Y {
		at := canvas.At(x, y)
		r, g, b, alpha := at.RGBA()

		if alpha > 0 {
			avg := (r + g + b) / 3

			newCanvas.Set(x, y, color.RGBA{
				R: uint8(uint32(tintColor.R) * avg >> 16),
				G: uint8(uint32(tintColor.G) * avg >> 16),
				B: uint8(uint32(tintColor.B) * avg >> 16),
				A: uint8(alpha),
			})
		}

		x++
		if x == max.X {
			x = 0
			y++
		}
	}

	return newCanvas
}
