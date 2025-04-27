import Synth from "./Synth.js";
import WebSocketHandler from "./WebSocketHandler.js";
import UserInteraction from "./UserInteraction.js";
import RTPMusicController from "./RTPMusicController.js";

// Initialize music controller and synth
const musicController = new RTPMusicController();
musicController.set_current_octave(3);
const synth = new Synth(musicController);

// Get DOM elements
const startAudioButton = document.getElementById('startAudio');
const sampleButtons = document.querySelectorAll('.sample-btn');
const xyPad = document.getElementById('xy-pad');
const touchMarker = document.getElementById('touch-marker');
const cutoffValueDisplay = document.getElementById('cutoff-value');
const resonanceValueDisplay = document.getElementById('resonance-value');

// Track touch/click state
let isInteracting = false;

// Audio start/stop button handler
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

// Sample selection button handlers
sampleButtons.forEach(button => {
    button.addEventListener('click', () => {
        // Extract sample set name from button ID
        const sampleSetName = button.id.replace('sample-', '');
        
        // Change the sample set
        if (synth.changeSampleSet(sampleSetName)) {
            // Update button styles
            sampleButtons.forEach(btn => btn.classList.remove('active'));
            button.classList.add('active');
            
            console.log(`Switched to sample set: ${sampleSetName}`);
        }
    });
});

// Function to handle XY-pad interaction
function handleXYInteraction(clientX, clientY) {
    const rect = xyPad.getBoundingClientRect();
    
    // Calculate normalized coordinates (0-1)
    const normalizedX = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    const normalizedY = Math.max(0, Math.min(1, (clientY - rect.top) / rect.height));
    
    // Position the touch marker
    touchMarker.style.left = `${clientX - rect.left}px`;
    touchMarker.style.top = `${clientY - rect.top}px`;
    touchMarker.style.opacity = '1';
    
    // Update filter parameters
    const cutoffFreq = synth.setFilterFrequency(normalizedY);
    const resonance = synth.setResonance(normalizedX);
    
    // Update display values
    cutoffValueDisplay.textContent = `Cutoff: ${cutoffFreq} Hz`;
    resonanceValueDisplay.textContent = `Resonance: ${resonance}`;
}

// XY-pad mouse/touch event handlers
xyPad.addEventListener('mousedown', (e) => {
    isInteracting = true;
    handleXYInteraction(e.clientX, e.clientY);
});

xyPad.addEventListener('mousemove', (e) => {
    if (isInteracting) {
        handleXYInteraction(e.clientX, e.clientY);
    }
});

xyPad.addEventListener('mouseup', () => {
    isInteracting = false;
    touchMarker.style.opacity = '0';
});

xyPad.addEventListener('mouseleave', () => {
    isInteracting = false;
    touchMarker.style.opacity = '0';
});

// Touch events for mobile devices
xyPad.addEventListener('touchstart', (e) => {
    e.preventDefault(); // Prevent scrolling
    isInteracting = true;
    handleXYInteraction(e.touches[0].clientX, e.touches[0].clientY);
});

xyPad.addEventListener('touchmove', (e) => {
    e.preventDefault(); // Prevent scrolling
    if (isInteracting) {
        handleXYInteraction(e.touches[0].clientX, e.touches[0].clientY);
    }
});

xyPad.addEventListener('touchend', () => {
    isInteracting = false;
    touchMarker.style.opacity = '0';
});

xyPad.addEventListener('touchcancel', () => {
    isInteracting = false;
    touchMarker.style.opacity = '0';
});

function processSocketData(data) {
    console.log('Received control change callback:', data);
    
    // Handle note off message
    if(data.control >= 12) {
        synth.playNoteOff();
        return;
    }
    
    // Update music controller with received data
    musicController.set_current_root(data.control);
    musicController.set_current_chord(data.value);
    
    // Log the current harmony
    const rootNoteName = musicController.get_current_root_note_name();
    const chordName = musicController.get_chord_name();
    console.log(`Playing harmony: ${rootNoteName} ${chordName}`);
    
    // Generate a position for the note based on the current pad dimensions
    const padRect = xyPad.getBoundingClientRect();
    const x = Math.random() * padRect.width;
    
    // Flash the touch marker briefly at a random position
    touchMarker.style.left = `${x}px`;
    touchMarker.style.top = `${padRect.height / 2}px`;
    touchMarker.style.opacity = '1';
    touchMarker.style.backgroundColor = '#00ff00';
    
    // Play the note
    synth.playNoteOnFromPosition(x);
    
    // Reset the marker after a short delay
    setTimeout(() => {
        touchMarker.style.opacity = '0';
        touchMarker.style.backgroundColor = '#00aaff';
    }, 200);
}

const webSocketHandler = new WebSocketHandler(`http://${window.location.hostname}:5000`, processSocketData);
const userInteraction = new UserInteraction(synth);


userInteraction.init();
