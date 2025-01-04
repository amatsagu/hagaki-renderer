package config

const DEFAULT_DYE_COLOR uint32 = 8289918

const (
	CDN_PATH          = "/home/amatsagu/Desktop/kikuri/cdn/"
	CHARACTER_IMAGE_X = 245
	CHARACTER_IMAGE_Y = 370
	CARD_MAX_X        = 303
	CARD_MAX_Y        = 428
)

type FrameType uint8

const (
	DEFAULT_FRAME FrameType = iota
	BETA_FRAME
	EDO_HIGAN_FRAME
)

var FrameTable = map[FrameType]FrameDetails{
	DEFAULT_FRAME: {
		Name:        "default",
		StaticModel: false,
		MaskModel:   true,
		Width:       245,
		Height:      370,
	},
	BETA_FRAME: {
		Name:        "beta",
		StaticModel: false,
		MaskModel:   true,
		Width:       251,
		Height:      376,
	},
	EDO_HIGAN_FRAME: {
		Name:        "edo-higan",
		StaticModel: true,
		MaskModel:   true,
		Width:       303,
		Height:      428,
	},
}

type FrameDetails struct {
	Name        string
	StaticModel bool // Static models cannot be dyed.
	MaskModel   bool // Mask models are dyable. Can be combined with static model, making only parts of frame dyable.
	Width       int
	Height      int
}
