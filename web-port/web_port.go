package main

import (
	"bytes"
	"context"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"strings"

	dapr "github.com/dapr/go-sdk/client"
)

func daprClientSend(image []byte, w http.ResponseWriter) {
	ctx := context.Background()

	// create the client
	client, err := dapr.NewClient()
	if err != nil {
		panic(err)
	}

	content := &dapr.DataContent{
		ContentType: "text/plain",
		Data:        image,
	}
	resp, err := client.InvokeMethodWithContent(ctx, "image-api-go", "/api/image", "post", content)
	if err != nil {
		panic(err)
	}
	log.Printf("dapr-wasmedge-go method api/image has invoked, response: %s", string(resp))
	fmt.Printf("Image classify result: %q\n", resp)
	w.WriteHeader(http.StatusOK)
	fmt.Fprintf(w, "%s", string(resp))
}

func httpClientSend(image []byte, w http.ResponseWriter) {
	client := &http.Client{}
	println("httpClientSend ....")

	// Dapr api format: http://localhost:<daprPort>/v1.0/invoke/<appId>/method/<method-name>
	var uri string
	uri = "http://localhost:3503/v1.0/invoke/image-api-wasi-socket-rs/method/image"
	println("uri: ", uri)
	
	req, err := http.NewRequest("POST", uri, bytes.NewBuffer(image))

	if err != nil {
		panic(err)
	}
	resp, err := client.Do(req)
	if err != nil {
		panic(err)
	}
	println(resp)

	defer resp.Body.Close()
	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		panic(err)
	}

	res := string(body)
	
	if strings.Contains(res, "Max bytes limit exceeded") {
		res = "ImageTooLarge"
	}
	fmt.Printf("Image classify result: %q\n", res)
	w.WriteHeader(http.StatusOK)
	fmt.Fprintf(w, "%s", res)
}

func imageHandler(w http.ResponseWriter, r *http.Request) {
	println("imageHandler ....")
	body, err := ioutil.ReadAll(r.Body)

	if err != nil {
		println("error: ", err.Error())
		panic(err)
	}
	api := r.Header.Get("api")
	if api == "go" {
		daprClientSend(body, w)
	} else {
		httpClientSend(body, w)
	}
}

func main() {
	fs := http.FileServer(http.Dir("./static"))
	http.Handle("/", http.StripPrefix("/", fs))
	http.HandleFunc("/api/hello", imageHandler)
	println("listen to 8080 ...")
	log.Fatal(http.ListenAndServe(":8080", nil))
}
