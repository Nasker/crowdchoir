export default class UserInteraction {
    constructor(synth) {
        this.synth = synth;
        this.init();
    }

    init() {
        window.addEventListener("click", (event) => this.handleMouseClick(event));
        window.addEventListener("mousemove", (event) => this.handleMouseMove(event));
    }

    handleMouseMove(event) {
        this.synth.setFilterFrequency(event.clientY);
    }

    handleMouseClick(event) {
        this.synth.playNoteFromPosition(event.clientX);
    }
}