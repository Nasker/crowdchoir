export default class WebSocketHandler {
    constructor(url) {
        this.socket = io(url, { transports: ['websocket'] });

        this.socket.on('connect', function(){
            console.log("Connected to the server");
        });

        this.socket.on('control_change', function(data){
            console.log('Received control change:', data);
        });
    }

    sendEvent(eventName, data) {
        this.socket.emit(eventName, data);
    }
}