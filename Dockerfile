FROM alpine:3
COPY mago /usr/local/bin/mago
ENTRYPOINT ["/usr/local/bin/mago"]
