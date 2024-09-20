import Synth from "./Synth.js";
import WebSocketHandler from "./WebSocketHandler.js";
import UserInteraction from "./UserInteraction.js";
import RTPMusicController  from "./RTPMusicController.js";


const musicController = new RTPMusicController();
musicController.up_octave()
musicController.up_octave()
musicController.up_octave()

const synth = new Synth(musicController);

function processSocketData(data) {
    console.log('Received control change callback:', data);
    musicController.set_current_root(data.control)
    musicController.set_current_chord(data.value)
    console.log(musicController.get_current_root_note_name());
    console.log(musicController.get_chord_name());
    //get a random x and y value between the window width and height
    //const x = Math.random() * window.innerWidth;
    //const y = Math.random() * window.innerHeight;
    //synth.playNoteFromPosition(x, y);
}

const webSocketHandler = new WebSocketHandler(`http://${window.location.hostname}:5000`,
                                                processSocketData);
const userInteraction = new UserInteraction(synth);


userInteraction.init();
