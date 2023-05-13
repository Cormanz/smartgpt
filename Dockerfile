FROM rust

# install tini to capture SIGINTs properly
RUN apt-get update && apt-get install -y tini

WORKDIR /smartgpt

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["cargo", "run", "--release"]