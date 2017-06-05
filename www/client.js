// Initialize WebSocket connetion without waiting for the DOM to
// be ready. I don't know if that's actually a good idea, but
// whatevs.
let socket = new WebSocket('ws://localhost:6767/api/client');
socket.onmessage = function(event) {
    console.log('socket event: ', event);
};

socket.onerror = function(error) {
    console.error(error);
};

socket.onclose = function() {
    console.log('Socket closed I guess');

    // TODO: Re-open the connection, if possible.
};

// Callback for "Feed Me" button. Sends a message to the backend notifying that
// a hippo has been fed.
function feedMe() {
    socket.send(JSON.stringify({
        event: 'feed-me',
        amount: 1,
    }));
}

// Initialize Vue.js with some info I guess.
let app = new Vue({
    el: '#app',
    data: {
        message: 'Hello Vue!'
    }
});
