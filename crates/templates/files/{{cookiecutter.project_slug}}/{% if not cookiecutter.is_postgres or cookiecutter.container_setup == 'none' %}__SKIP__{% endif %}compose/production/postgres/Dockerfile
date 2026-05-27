FROM postgres:18-alpine

COPY maintenance /usr/local/bin/maintenance
RUN chmod +x /usr/local/bin/maintenance/* && mv /usr/local/bin/maintenance/* /usr/local/bin/ && rmdir /usr/local/bin/maintenance
