FROM gbbabarros/rollups-cli:1.0.1

RUN apk add --no-cache jq

COPY deploy.sh /deploy.sh
RUN chmod +x /deploy.sh

ENTRYPOINT ["/deploy.sh"]
