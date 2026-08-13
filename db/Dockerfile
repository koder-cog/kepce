FROM postgres:15

# pgBackRest ve diğer gerekli araçları kur
RUN apt-get update && \
    apt-get install -y pgbackrest && \
    rm -rf /var/lib/apt/lists/*

# pgBackRest yedek dizinlerini oluştur ve izinlerini ayarla
RUN mkdir -p /opt/kepce/backups/pgbackrest && \
    chown postgres:postgres /opt/kepce/backups/pgbackrest && \
    chmod 750 /opt/kepce/backups/pgbackrest

# Log dizini
RUN mkdir -p /var/log/pgbackrest && \
    chown postgres:postgres /var/log/pgbackrest

# Docker entrypoint initialization scripts
COPY scripts/init-db.sh /docker-entrypoint-initdb.d/00_init-db.sh
COPY migrations /docker-entrypoint-initdb.d/migrations
COPY seeds/prod /docker-entrypoint-initdb.d/seeds/prod

RUN chmod +x /docker-entrypoint-initdb.d/00_init-db.sh
