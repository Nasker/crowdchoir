import RTPChordMatrix from './RTPChordMatrix.js';

// Notes in the C blues scale
const bluesScale = ["C4", "Eb4", "F4", "F#4", "G4", "Bb4", "C5"];

// Create a single Tone.Synth object
const synth = new Tone.Synth();

// Create a feedback delay effect
const feedbackDelay = new Tone.FeedbackDelay("8n", 0.5).toDestination();

// Create a lowpass filter
const filter = new Tone.Filter({
    type: 'lowpass',
    frequency: 350,
    Q: 1
});

synth.connect(filter);
filter.connect(feedbackDelay);

function playNoteFromMousePosition(x, y) {
    const noteIndex = Math.floor((x / window.innerWidth) * bluesScale.length);
    const note = bluesScale[noteIndex];
    const minFreq = Math.log(20);
    const maxFreq = Math.log(20000);
    const scaleY = 1 - (y / window.innerHeight);
    const frequency = Math.exp(scaleY * (maxFreq - minFreq) + minFreq);
    filter.frequency.value = frequency;
    const time = Tone.now();
    synth.triggerAttackRelease(note, "8n", time);
}

// Function to handle mouse click
function handleMouseClick(event) {
    const x = event.clientX;
    const y = event.clientY;

    // Play note based on mouse position
    playNoteFromMousePosition(x, y);
}

// Add event listener for mouse click
window.addEventListener("click", handleMouseClick);