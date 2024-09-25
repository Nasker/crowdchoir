import Synth from "./Synth.js";
import WebSocketHandler from "./WebSocketHandler.js";
import UserInteraction from "./UserInteraction.js";
import RTPMusicController  from "./RTPMusicController.js";


const musicController = new RTPMusicController();
musicController.set_current_octave(4);
const synth = new Synth(musicController);
const startAudioButton = document.getElementById('startAudio');

startAudioButton.addEventListener('click', () => {
    if (Tone.context.state !== 'running') {
        Tone.start();
        startAudioButton.textContent = 'Stop Audio';
        startAudioButton.classList.remove('inactive');
        startAudioButton.classList.add('active');
    } else {
        Tone.context.close();
        startAudioButton.textContent = 'Start Audio';
        startAudioButton.classList.remove('active');
        startAudioButton.classList.add('inactive');
    }
});

function processSocketData(data) {
    console.log('Received control change callback:', data);
    if(data.control >= 12){
        synth.playNoteOff()
        return
    }
    musicController.set_current_root(data.control)
    musicController.set_current_chord(data.value)
    console.log(musicController.get_current_root_note_name());
    console.log(musicController.get_chord_name());
    const x = Math.random() * window.innerWidth;
    synth.playNoteOnFromPosition(x);
}

const webSocketHandler = new WebSocketHandler(`http://${window.location.hostname}:5000`, processSocketData);
const userInteraction = new UserInteraction(synth);


userInteraction.init();
