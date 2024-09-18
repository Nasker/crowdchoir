export default class UserInteraction {
    constructor(synth) {
        this.synth = synth;
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