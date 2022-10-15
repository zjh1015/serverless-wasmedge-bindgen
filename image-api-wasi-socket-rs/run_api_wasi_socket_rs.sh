dapr run --app-id image-api-wasi-socket-rs \
         --app-protocol http \
         --app-port 9005 \
         --dapr-http-port 3503 \
         --components-path ../config \
         --log-level debug \
	 wasmedge-tensorflow ./target/wasm32-wasi/release/image-api-wasi-socket-rs.wasm
