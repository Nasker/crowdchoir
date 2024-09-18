export default class WebSocketHandler {
    constructor(url) {
        this.socket = io(url, { transports: ['websocket'] });
        this.socket.on('connect', () => {
            console.log("Connected to server");
        });
        this.socket.on('control_change', (data) => {
            console.log('Received control change:', data);
        });
    }

    sendEvent(eventName, data) {
        this.socket.emit(eventName, data);
    }
}