// Notes in the C blues scale
const bluesScale = ["C4", "Eb4", "F4", "F#4", "G4", "Bb4", "C5"];

// Create a single Tone.Synth object
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

// Connect the synth to the filter, and then the filter to the feedback delay
synth.connect(filter);
filter.connect(feedbackDelay);

function playNoteFromMousePosition(x, y) {
    // Normalize x to select a note from the blues scale
    const noteIndex = Math.floor((x / window.innerWidth) * bluesScale.length);
    const note = bluesScale[noteIndex];

    // Map y to control the filter frequency in a logarithmic scale and reverse the direction
    const minFreq = Math.log(20); // Minimum frequency (20 Hz) in a logarithmic scale
    const maxFreq = Math.log(20000); // Maximum frequency (20000 Hz) in a logarithmic scale
    const scaleY = 1 - (y / window.innerHeight); // Reverse the direction
    const frequency = Math.exp(scaleY * (maxFreq - minFreq) + minFreq); // Convert back to a linear scale

    // Set the frequency of the filter
    filter.frequency.value = frequency;

    // Trigger the note as soon as possible
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