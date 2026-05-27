FROM traefik:v3.3

COPY traefik.yml /etc/traefik/traefik.yml
RUN mkdir -p /etc/traefik/acme && chown -R 1000:1000 /etc/traefik/acme

EXPOSE 80 443
