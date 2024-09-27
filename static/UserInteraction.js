export default class UserInteraction {
    constructor(synth) {
        this.synth = synth;
        this.isRunning = false;
        this.init();
    }

    init() {
        window.addEventListener("mousemove", (event) => this.handleMouseMove(event));
        document.getElementById("start_demo").addEventListener("click", (e) => this.toggleGyro(e));
    }

    toggleGyro(e) {
        e.preventDefault();
        const demoButton = e.target;
        if (this.isRunning) {
            window.removeEventListener("deviceorientation", this.handleOrientation.bind(this));
            demoButton.innerHTML = "Start demo";
            demoButton.classList.remove('btn-danger');
            demoButton.classList.add('btn-success');
            this.isRunning = false;
        } else {
            if (typeof DeviceOrientationEvent.requestPermission === "function") {
                DeviceOrientationEvent.requestPermission()
                    .then(permissionState => {
                        if (permissionState === "granted") {
                            window.addEventListener("deviceorientation", this.handleOrientation.bind(this));
                        }
                    })
                    .catch(console.error);
            } else {
                window.addEventListener("deviceorientation", this.handleOrientation.bind(this));
                console.log("DeviceOrientation Added");
            }
            demoButton.innerHTML = "Stop demo";
            demoButton.classList.remove('btn-success');
            demoButton.classList.add('btn-danger');
            this.isRunning = true;
        }
    }

    handleMouseMove(event) {
        this.synth.setFilterFrequency(event.clientY);
    }

    handleOrientation(event) {
        if (event.beta != null) {
            let y_range = (event.beta + 180) * (window.innerHeight / 360);
            this.synth.setFilterFrequency(y_range);
        }
    }
}
