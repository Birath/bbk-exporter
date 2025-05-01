FROM alpine AS build

RUN apk add --no-cache --repository=http://dl-cdn.alpinelinux.org/alpine/edge/main \
    g++ \
    make \ 
    git \
    rust \
    cargo

ADD https://github.com/dotse/bbk.git /bbk

RUN cd bbk/src/cli && \
    make -j $(nproc)

ADD . /bbk_exporter
RUN cd /bbk_exporter && \
    cargo build --release && \
    cargo install --path . --root /

FROM alpine AS bbk-exporter

RUN apk add --no-cache \
    libstdc++ \
    libgcc

COPY --from=build /bbk/src/cli/cli /bin/bbk
COPY --from=build /bin/bbk_exporter /bin/bbk_exporter

ENTRYPOINT [ "/bin/bbk_exporter", "--bbk", "/bin/bbk" ]






