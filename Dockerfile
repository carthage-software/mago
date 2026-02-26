FROM scratch
COPY mago /usr/local/bin/mago
ENTRYPOINT ["/usr/local/bin/mago"]
