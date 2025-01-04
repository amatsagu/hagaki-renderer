package render

import (
	"context"
	"errors"
	"fmt"
	"image"
	"image/draw"
	"image/png"
	"maestro/config"
	"maestro/logger"
	"os"
)

type RenderRequestData struct {
	ID          uint32           `json:"id"`
	Dye         uint32           `json:"dye,omitempty"`
	FrameType   config.FrameType `json:"frame,omitempty"`
	CustomImage bool             `json:"custom_image,omitempty"` // When true - render using player custom card image, request's ID points then to card ID
	Glow        bool             `json:"glow,omitempty"`
	OffsetX     int              `json:"offset_x,omitempty"`
	OffsetY     int              `json:"offset_y,omitempty"`
}

func RenderCard(ctx context.Context, rrd RenderRequestData) (*image.RGBA, error) {
	if int(rrd.FrameType) > len(config.FrameTable) {
		return nil, errors.New("requested frame doesn't exist")
	}

	charImgPath := fmt.Sprintf("%s/public/character/%d.png", config.CDN_PATH, rrd.ID)
	if rrd.CustomImage {
		charImgPath = fmt.Sprintf("%s/public/custom-character/%d.png", config.CDN_PATH, rrd.ID)
	}

	charImgBuf, err := os.OpenFile(charImgPath, os.O_RDONLY, 0755)
	if err != nil {
		return nil, fmt.Errorf("requested character (ID = %d) has no valid png image to use", rrd.ID)
	}

	characterImage, err := png.Decode(charImgBuf)
	if err != nil {
		logger.Error.Printf("Failed to decode character [%+v] image: %s\n", rrd, err.Error())
		return nil, errors.New("failed to decode character image - check server logs")
	}
	defer charImgBuf.Close()

	ctxErr := ctx.Err()
	if ctxErr != nil {
		if ctxErr == context.DeadlineExceeded {
			logger.Warn.Printf("Can't keep up! Card render took over 5 seconds!")
			return nil, errors.New("request took too long to process")
		}
		logger.Error.Println("Context failure, failed at card render: ", ctxErr.Error())
		return nil, errors.New("context failure - something went wrong, check server logs")
	}

	frameDetails := config.FrameTable[rrd.FrameType]
	offsetX := (frameDetails.Width - config.CHARACTER_IMAGE_X) / 2
	offsetY := (frameDetails.Height - config.CHARACTER_IMAGE_Y) / 2
	charImgOffsetX := rrd.OffsetX + offsetX
	charImgOffsetY := rrd.OffsetY + offsetY
	canvas := image.NewRGBA(image.Rectangle{
		image.Point{},
		image.Point{
			frameDetails.Width,
			frameDetails.Height,
		},
	})

	draw.Draw(
		canvas,
		image.Rectangle{
			image.Point{
				charImgOffsetX,
				charImgOffsetY,
			},
			image.Point{
				config.CHARACTER_IMAGE_X + charImgOffsetX,
				config.CHARACTER_IMAGE_Y + charImgOffsetY,
			},
		},
		characterImage,
		image.Point{},
		draw.Src,
	)

	ctxErr = ctx.Err()
	if ctxErr != nil {
		if ctxErr == context.DeadlineExceeded {
			logger.Warn.Printf("Can't keep up! Card render took over 5 seconds!")
			return nil, errors.New("request took too long to process")
		}
		logger.Error.Println("Context failure, failed at card render: ", ctxErr.Error())
		return nil, errors.New("context failure - something went wrong, check server logs")
	}

	if frameDetails.MaskModel {
		var (
			maskFrameBuf *os.File
			err          error
		)

		if rrd.Glow {
			maskFrameBuf, err = os.OpenFile(fmt.Sprintf("%s/private/frame/%s/glow-mask.png", config.CDN_PATH, frameDetails.Name), os.O_RDONLY, 0755)
		} else {
			maskFrameBuf, err = os.OpenFile(fmt.Sprintf("%s/private/frame/%s/mask.png", config.CDN_PATH, frameDetails.Name), os.O_RDONLY, 0755)
		}

		if err != nil {
			return nil, errors.New("requested frame's mask doesn't exist")
		}

		maskFrameImage, err := png.Decode(maskFrameBuf)
		if err != nil {
			logger.Error.Printf("Failed to decode frame's mask [%+v] image: %s\n", frameDetails, err.Error())
			return nil, errors.New("failed to decode frame's mask image - check server logs")
		}
		defer maskFrameBuf.Close()

		ctxErr = ctx.Err()
		if ctxErr != nil {
			if ctxErr == context.DeadlineExceeded {
				logger.Warn.Printf("Can't keep up! Card render took over 5 seconds!")
				return nil, errors.New("request took too long to process")
			}
			logger.Error.Println("Context failure, failed at card render: ", ctxErr.Error())
			return nil, errors.New("context failure - something went wrong, check server logs")
		}

		draw.Draw(
			canvas,
			image.Rectangle{
				image.Point{
					rrd.OffsetX,
					rrd.OffsetY,
				},
				image.Point{
					rrd.OffsetX + frameDetails.Width,
					rrd.OffsetY + frameDetails.Height,
				},
			},
			Recolor(maskFrameImage, ColorValueToRGBA(rrd.Dye)),
			image.Point{},
			draw.Over,
		)
	}

	if frameDetails.StaticModel {
		var (
			staticFrameBuf *os.File
			err            error
		)

		if rrd.Glow {
			staticFrameBuf, err = os.OpenFile(fmt.Sprintf("%s/private/frame/%s/glow-static.png", config.CDN_PATH, frameDetails.Name), os.O_RDONLY, 0755)
		} else {
			staticFrameBuf, err = os.OpenFile(fmt.Sprintf("%s/private/frame/%s/static.png", config.CDN_PATH, frameDetails.Name), os.O_RDONLY, 0755)
		}

		if err != nil {
			return nil, errors.New("requested frame's static model doesn't exist")
		}

		staticFrameImage, err := png.Decode(staticFrameBuf)
		if err != nil {
			logger.Error.Printf("Failed to decode frame's static model [%+v] image: %s\n", frameDetails, err.Error())
			return nil, errors.New("failed to decode frame's static model image - check server logs")
		}
		defer staticFrameBuf.Close()

		ctxErr = ctx.Err()
		if ctxErr != nil {
			if ctxErr == context.DeadlineExceeded {
				logger.Warn.Printf("Can't keep up! Card render took over 5 seconds!")
				return nil, errors.New("request took too long to process")
			}
			logger.Error.Println("Context failure, failed at card render: ", ctxErr.Error())
			return nil, errors.New("context failure - something went wrong, check server logs")
		}

		draw.Draw(
			canvas,
			image.Rectangle{
				image.Point{
					rrd.OffsetX,
					rrd.OffsetY,
				},
				image.Point{
					rrd.OffsetX + frameDetails.Width,
					rrd.OffsetY + frameDetails.Height,
				},
			},
			staticFrameImage,
			image.Point{},
			draw.Over,
		)
	}

	return canvas, nil
}
