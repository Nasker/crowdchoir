import Synth from "./Synth.js";
import WebSocketHandler from "./WebSocketHandler.js";
import UserInteraction from "./UserInteraction.js";

const synth = new Synth();
const webSocketHandler = new WebSocketHandler('http://127.0.0.1:5000');
const userInteraction = new UserInteraction(synth);

userInteraction.init();
