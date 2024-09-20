export default class WebSocketHandler {
    constructor(url, onControlChange) {
        this.socket = io(url, { transports: ['websocket'] });

        this.socket.on('connect', function(){
            console.log("Connected to the server");
        });

        this.socket.on('control_change', function(data){
            onControlChange(data);
        });

        this.socket.on('disconnect', function(){
            console.log("Disconnected from the server");
        });

        this.socket.on('error', function(err){
            console.error("WebSocket error:", err);
        });

        this.socket.on('reconnect_attempt', () => {
            console.log("Attempting to reconnect...");
        });
    }

    sendEvent(eventName, data) {
        this.socket.emit(eventName, data);
    }
}