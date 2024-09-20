import Synth from "./Synth.js";
import WebSocketHandler from "./WebSocketHandler.js";
import UserInteraction from "./UserInteraction.js";

const synth = new Synth();
const webSocketHandler = new WebSocketHandler(`http://${window.location.hostname}:5000`);
const userInteraction = new UserInteraction(synth);

userInteraction.init();
