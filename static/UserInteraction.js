export default class UserInteraction {
    constructor(synth) {
        this.synth = synth;
        this.isRunning = false;
        this.init();
    }

    init() {
        // Add both mouse and device orientation event listeners
        window.addEventListener("mousemove", (event) => this.handleMouseMove(event));

        // Check for DeviceOrientation support
        if (window.DeviceOrientationEvent) {
            document.getElementById("start_demo").addEventListener("click", (e) => this.toggleGyro(e));
        }
    }

    // Start/Stop gyroscope events on button click
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
            // Request permission on iOS devices
            if (typeof DeviceOrientationEvent.requestPermission === "function") {
                DeviceOrientationEvent.requestPermission()
                    .then(permissionState => {
                        if (permissionState === "granted") {
                            window.addEventListener("deviceorientation", this.handleOrientation.bind(this));
                        }
                    })
                    .catch(console.error);
            } else {
                // For non-iOS devices, add event listener directly
                window.addEventListener("deviceorientation", this.handleOrientation.bind(this));
            }

            demoButton.innerHTML = "Stop demo";
            demoButton.classList.remove('btn-success');
            demoButton.classList.add('btn-danger');
            this.isRunning = true;
        }
    }

    // Handle mouse movement
    handleMouseMove(event) {
        this.synth.setFilterFrequency(event.clientY);
    }

    // Handle device orientation (gyroscope data)
    handleOrientation(event) {
        if (event.beta != null) {
            // Map beta (tilt front/back) to frequency filter range
            let y_range = (event.beta + 180) * (window.innerHeight / 360);
            this.synth.setFilterFrequency(y_range);
        }
    }
}
