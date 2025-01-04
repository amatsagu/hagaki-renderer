package logger

import (
	"log"
	"os"
)

var (
	Info  *log.Logger
	Warn  *log.Logger
	Error *log.Logger
)

func InitLogger() {
	Info = log.New(os.Stdout, "\033[0mINFO: ", log.LstdFlags|log.Lshortfile)
	Warn = log.New(os.Stdout, "\033[33mWARN: ", log.LstdFlags|log.Lshortfile)
	Error = log.New(os.Stdout, "\033[31mERROR: ", log.LstdFlags|log.Lshortfile)
}
