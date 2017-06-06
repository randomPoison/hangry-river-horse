let app = new Vue({
    el: '#app',
    data: {
        scores: [],
    }
});

let socket = new WebSocket('ws://localhost:6767/api/host-stream');
socket.onmessage = (event) => {
    console.log('payload: ', event);

    // TODO: Do some validation on the payload data I guess.
    let payload = JSON.parse(event.data);

    // Convert object into array.
    // TODO: This should be done server-side, rather than needing the client to do it.
    let scores = [];
    for (id in payload.scores) {
        scores.push({
            player: id,
            score: payload.scores[id],
        });
    }
    console.log('scores:', scores);
    app.scores = scores;
};
