package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
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

	port := ":8080"
	fmt.Printf("Listening on: %s\n", port)
	http.ListenAndServe(port, nil)
}
