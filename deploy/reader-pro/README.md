# Production deployment

`reader-pro-restored` runs beside the existing Reader container and binds only
to `127.0.0.1:18088`. The host Nginx terminates TLS for `read.medwarp.cn`.

Required server-side files that are deliberately not committed:

- `.env`: authentication and deployment configuration, mode 600;
- `storage/`: imported Reader data, owned by UID/GID 10001;
- `logs/`: application logs, owned by UID/GID 10001;
- `reader-4.0.7.jar`: output of `scripts/build.ps1`.

Build and start on the server:

```bash
cd /opt/reader-pro-restored
docker compose config --quiet
docker compose build
docker compose up -d
curl -fsS http://127.0.0.1:18088/
```

