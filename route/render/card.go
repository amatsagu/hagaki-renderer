package route

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"image/png"
	"maestro/config"
	"maestro/logger"
	"maestro/render"
	"net/http"
	"time"
)

// It requires hash to be in base64, example:
// http://127.0.0.1:8899/render/card/eyJpZCI6MSwiZnJhbWUiOjJ9
func CardHandler(w http.ResponseWriter, r *http.Request) {
	hash64 := r.PathValue("hash")
	if hash64 == "" {
		http.Error(w, "unauthorized", http.StatusUnauthorized)
		return
	}

	decoded, err := base64.StdEncoding.DecodeString(hash64)
	if err != nil {
		http.Error(w, "unauthorized", http.StatusUnauthorized)
		return
	}

	logger.Warn.Println(string(decoded))
	rrd := render.RenderRequestData{}
	err = json.Unmarshal(decoded, &rrd)
	if err != nil {
		http.Error(w, "bad request data", http.StatusBadRequest)
		return
	}

	if int(rrd.FrameType) > len(config.FrameTable) {
		http.Error(w, "requested frame doesn't exist", http.StatusBadRequest)
		return
	}

	startedAt := time.Now()
	ctx, cancel := context.WithTimeout(r.Context(), 5*time.Second)
	defer cancel()

	canvas, err := render.RenderCard(ctx, rrd)
	if err != nil {
		http.Error(w, err.Error(), http.StatusUnprocessableEntity)
		return
	}

	w.Header().Set("X-Processing-Time", time.Since(startedAt).String())
	w.Header().Set("Content-Type", "image/png")
	enc := &png.Encoder{
		CompressionLevel: png.NoCompression,
	}

	if err = enc.Encode(w, canvas); err != nil {
		logger.Error.Println("Failed to encode finished card render to response: ", err.Error())
	}
}
