FROM node:22-alpine as ui-builder
WORKDIR /usr/src/
COPY . ./
RUN cd frontend && npm install && npm run build

FROM messense/rust-musl-cross:x86_64-musl AS builder
WORKDIR /usr/src/
COPY . ./
COPY --from=ui-builder /usr/src/frontend/dist /usr/src/frontend/dist
RUN BUILD_UI_ENABLED=0 cargo build --release --target=x86_64-unknown-linux-musl

# Bundle Stage
FROM alpine
VOLUME [ "/data" ]
RUN apk add --update bash
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/server /app/server
WORKDIR /data
EXPOSE 3001
CMD ["/app/server"]