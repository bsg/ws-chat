document.addEventListener("DOMContentLoaded", function (event) {
    console.log("on load");
    var divChat = document.getElementById("chat");
    var txtInput = document.getElementById("msgInput");
    app(divChat, txtInput);
});
function app(divChat, txtInput) {
    txtInput.addEventListener('keydown', (function (event) {
        if (event.key == "Enter") {
            sendMessage(txtInput.value);
            txtInput.value = "";
        }
    }));
    // TODO reconnect
    var ws = new WebSocket("ws://" + location.host + "/stream");
    ws.addEventListener("open", function (event) {
        console.log("ws open");
        divChat.innerHTML += "<div class=\"oob\">Connected</div>";
    });
    ws.addEventListener("close", function (event) {
        console.log("ws closed");
        divChat.innerHTML += "<div class=\"oob\">Disconnected</div>";
    });
    ws.addEventListener("message", function (event) {
        console.log(event.data);
        renderMessage(JSON.parse(event.data));
    });
    function sendMessage(message) {
        ws.send(message);
    }
    function renderMessage(message) {
        divChat.innerHTML += "<div class=\"msgItem\">" + message.username + ": " + message.body + "</div>";
        divChat.scrollTop = divChat.scrollHeight;
    }
}
