self.addEventListener('fetch', event => {
  event.respondWith(
    caches.match(event.request).then(cachedResponse => {
      const networkFetch = requestOverNetwork(event);
      // prioritize cached response over network
      return networkFetch.catch(_ => cachedResponse);
    }
    ).catch(function (reason) {
      console.error('ServiceWorker cache match failed: ', reason);
      const networkFetch = requestOverNetwork(event);
      return networkFetch;
    })
  );
});

function requestOverNetwork(event) {
  return fetch(event.request).then(response => {
    // update the cache with a clone of the network response
    const responseClone = response.clone();
    caches.open("xpress-calc").then(cache => {
      cache.put(event.request, responseClone);
    });
    return response;
  }).catch(function (reason) {
    console.error('ServiceWorker fetch failed: ', reason);
    return Promise.reject(reason);
  });
}
