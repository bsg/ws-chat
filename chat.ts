interface Message {
    username: String,
    body: String
}

document.addEventListener("DOMContentLoaded", function (event) {
    console.log("on load");
    var divChat = document.getElementById("chat") as HTMLElement;
    var txtInput = document.getElementById("msgInput") as HTMLInputElement;
    app(divChat, txtInput);
});

function app(divChat: HTMLElement, txtInput: HTMLInputElement) {
    txtInput.addEventListener('keydown', (event => {
        if (event.key == "Enter") {
            sendMessage(txtInput.value);
            txtInput.value = "";
        }
    }));

    // TODO reconnect
    var ws = new WebSocket("ws://" + location.host + "/stream");
    ws.addEventListener("open", (event) => {
        console.log("ws open")
        divChat.innerHTML += "<div class=\"oob\">Connected</div>";
    });
    ws.addEventListener("close", (event) => {
        console.log("ws closed")
        divChat.innerHTML += "<div class=\"oob\">Disconnected</div>";
    });
    ws.addEventListener("message", (event) => {
        console.log(event.data);
        renderMessage(JSON.parse(event.data));
    });

    function sendMessage(message: string) {
        ws.send(message);
    }

    function renderMessage(message: Message) {
        divChat.innerHTML += "<div class=\"msgItem\">" + message.username + ": " + message.body + "</div>";
        divChat.scrollTop = divChat.scrollHeight;
    }
}

