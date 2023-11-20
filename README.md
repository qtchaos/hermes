# Ziria
Blazing fast and memory-safe avatar service.

## Caching
To achieve blazing fast speeds and to not get rate-limited by Mojang, we apply the following caching technique for avatars:
* Store an optimized 8px * 8px image in Redis.
* Resize the optimized image on request.
* Renew the cache every 20 minutes.

## Reliability
To ensure reliability of the service, we plan to store avatars in long term storage, which is slower but more cost efficient than storing in memory. If Mojang' servers ever go down, your avatar can still be served if you've used the service before. Long term storage should be updated only if a request has been made and the last update of long term storage is greater than 24 hours.

## API
We currently provide three endpoints:
* `/avatar/{uuid}/{size}/{helm}`
* `/skin/{uuid}/{size}`
* `/skin/{uuid}`

### Avatar
To use the avatar service, you can use the `/avatar/{uuid}/{size}/{helm}` endpoint, `helm` being the overlay/helmet layer of the skin.

### Skin
To use the skin service, you can use either the `/skin/{uuid}/{size}` endpoint, or the `/skin/{uuid}`, the latter returns the original image without any resizing.