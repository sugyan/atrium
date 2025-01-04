## Example code for uploading a video

1. First, get a token by `com.atproto.server.getServiceAuth`.

2. Call uploadVideo against the video service (`video.bsky.app`) with the token.

3. Call `app.bsky.video.getJobStatus` against the video service as well.

(The same goes for `app.bsky.video.getUploadLimits`, which gets a token and calls the video service with it to get the data, but the process of checking this may be omitted.)

In Atrium:

- Since `AtpAgent` cannot process XRPC requests with the token obtained by `getServiceAuth`, we need to prepare a dedicated Client and create an `AtpServiceClient` that uses it.
- The `app.bsky.video.uploadVideo` endpoint is special (weird?) and requires special hacks such as adding query parameters to the request URL and modifying the response to match the schema.