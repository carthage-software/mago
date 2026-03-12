FROM alpine:3
RUN apk add git 
COPY mago /usr/local/bin/mago
ENTRYPOINT ["/usr/local/bin/mago"]
