# CrowdChoir captive-portal deployment (Raspberry Pi)

Turns a Raspberry Pi into a self-contained WiFi hotspot that:

1. Broadcasts an open `CrowdChoir` network (`hostapd`).
2. Hands out IPs and resolves **all** DNS to the Pi (`dnsmasq`) so phones show a
   captive portal and never drop the network.
3. Serves a splash page on **HTTP :80** that links to the app.
4. Serves the real app on **HTTPS :5000** (required for the gyroscope).

```
Phone ──WiFi──> Pi (10.0.0.1)
                ├── hostapd  → SSID "CrowdChoir"
                ├── dnsmasq  → DHCP + wildcard DNS
                ├── :80  HTTP  → splash (run_captive_portal)
                └── :5000 HTTPS → CrowdChoir app (run_server)
```

## Why the two-step (splash → browser)?

The captive-portal mini-browser (iOS CNA / Android webview) is sandboxed and
won't reliably run the gyroscope or audio. So the HTTP splash just says *"Tap to
Join"* and links out to the HTTPS app in the **real** browser, where everything
works. Users accept the self-signed cert warning once.

## 1. Install packages

```bash
sudo apt update
sudo apt install -y hostapd dnsmasq
sudo systemctl unmask hostapd
```

## 2. Give the Pi a static AP address

Add to `/etc/dhcpcd.conf`:

```
interface wlan0
static ip_address=10.0.0.1/24
nohook wpa_supplicant
```

## 3. Drop in the config files

```bash
sudo cp hostapd.conf  /etc/hostapd/hostapd.conf
sudo cp dnsmasq.conf  /etc/dnsmasq.conf
# Point hostapd at its config:
echo 'DAEMON_CONF="/etc/hostapd/hostapd.conf"' | sudo tee -a /etc/default/hostapd
```

Edit `interface=` in both files if your WiFi device isn't `wlan0` (`iw dev`).

## 4. Build & install the server

```bash
cargo build --release          # produces target/release/crowdchoir-server
sudo cp crowdchoir.service /etc/systemd/system/crowdchoir.service
sudo systemctl daemon-reload
sudo systemctl enable --now hostapd dnsmasq crowdchoir
```

## 5. Test

- Join the `CrowdChoir` WiFi from a phone.
- The captive-portal sheet should pop up showing **🎶 Join the Choir**.
- Tap **Tap to Join** → opens `https://choir.party:5000` in the browser.
- Accept the security warning once → audio + gyroscope work.

## Configuration (env vars, set in `crowdchoir.service`)

| Variable                 | Purpose                                              |
|--------------------------|------------------------------------------------------|
| `CROWDCHOIR_TLS=1`       | Serve the app over HTTPS (required for gyroscope).   |
| `CROWDCHOIR_CAPTIVE=1`   | Run the HTTP :80 captive splash.                     |
| `CROWDCHOIR_PUBLIC_URL`  | URL the splash links to (`https://choir.party:5000`).|
| `CROWDCHOIR_TLS_SANS`    | Extra cert SANs: hostname + AP IP (`choir.party,10.0.0.1`). |
| `PORT`                   | App HTTPS port (default 5000).                       |

## Notes & gotchas

- **Port 80** needs `CAP_NET_BIND_SERVICE` (already in the unit file) or root.
- The cert is **self-signed** — there is no way to avoid the one-time browser
  warning offline. `CROWDCHOIR_TLS_SANS` ensures it at least matches the
  hostname/IP so it isn't rejected outright.
- `address=/#/10.0.0.1` in `dnsmasq.conf` is the DNS spoof. Removing it disables
  the captive behaviour.
- iOS occasionally caches a "no portal" result; toggling WiFi off/on forces a
  re-probe.
