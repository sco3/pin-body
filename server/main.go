package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

func main() {
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		body, _ := io.ReadAll(r.Body)

		resp := map[string]string{
			"status":   "ok",
			"received": string(body),
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	})

	http.HandleFunc("/events", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/event-stream")
		w.Header().Set("Cache-Control", "no-cache")
		w.Header().Set("Connection", "keep-alive")

		flusher, ok := w.(http.Flusher)
		if !ok {
			http.Error(w, "Streaming unsupported!", http.StatusInternalServerError)
			return
		}

		fmt.Println("Client connected to SSE")

		for i := 0; i < 42; i++ {
			fmt.Fprintf(w, "data: Message %d at %s\n\n", i, time.Now().Format("15:04:05"))
			flusher.Flush()
			time.Sleep(2 * time.Second)
		}
	})

	port := ":8080"
	fmt.Printf("Listening on: %s\n", port)
	http.ListenAndServe(port, nil)
}
