package main

import (
	"flag"
	"maestro/logger"
	routeRender "maestro/route/render"
	"net/http"
)

func main() {
	logger.InitLogger()
	logger.Info.Println("Starting Maestro application...")

	// In real usage, it'll be prefixed with api and version, for example:
	// https://kikuri-bot
	http.HandleFunc("GET /render/card/{hash}", routeRender.CardHandler)

	var addr string
	flag.StringVar(&addr, "addr", "127.0.0.1:8899", "switches which address should be used")
	flag.Parse()

	logger.Info.Printf("Started application on %s.\n", addr)
	if err := http.ListenAndServe(addr, nil); err != nil {
		logger.Error.Panic(err)
	}
}
