# Docker Swarm File

```yaml
version: "3"

services:
  redis:
    image: docker.dragonflydb.io/dragonflydb/dragonfly
    deploy:
      mode: global
    ports:
      - "6379:6379"

  newsletter:
    image: ghcr.io/kunalsin9h/newsletter:latest
    deploy:
      mode: replicated
      replicas: 1
    ports:
      - "5000:5000"
    volumes:
      - ./production.yaml:/newsletter/configuration/production.yaml
```

The `production.yaml` is same as the [local.yaml](https://github.com/KunalSin9h/newsletter/blob/master/configuration/local.yaml) file, but with real productions values.

