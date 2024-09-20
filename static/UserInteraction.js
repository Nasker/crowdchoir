export default class UserInteraction {
    constructor(synth, musicController) {
        this.synth = synth;
        this.musicController = musicController;
        this.init();
    }

    init() {
        window.addEventListener("click", (event) => this.handleMouseClick(event));
    }

    handleMouseClick(event) {
        const x = event.clientX;
        const y = event.clientY;
        this.synth.playNoteFromPosition(x, y);
    }
}