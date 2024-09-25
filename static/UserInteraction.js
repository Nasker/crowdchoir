export default class UserInteraction {
    constructor(synth) {
        this.synth = synth;
        this.init();
    }

    init() {
        window.addEventListener("mousemove", (event) => this.handleMouseMove(event));
    }

    handleMouseMove(event) {
        this.synth.setFilterFrequency(event.clientY);
    }
}