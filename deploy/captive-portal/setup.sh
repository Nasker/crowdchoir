#!/usr/bin/env bash
# CrowdChoir captive-portal one-shot setup for Raspberry Pi (Debian/Raspberry Pi OS).
# Installs hostapd + dnsmasq, configures the AP + DNS spoof, installs the
# systemd service, and starts everything. Re-runnable (idempotent).
#
# Usage:
#   sudo ./setup.sh
#
# Override defaults via env vars, e.g.:
#   sudo WIFI_IFACE=wlan1 AP_IP=192.168.50.1 ./setup.sh

set -euo pipefail

# ── Tunables ────────────────────────────────────────────────────────────────
WIFI_IFACE="${WIFI_IFACE:-wlan0}"
AP_IP="${AP_IP:-10.0.0.1}"
SERVICE_USER="${SERVICE_USER:-pi}"
APP_DIR="${APP_DIR:-/home/${SERVICE_USER}/crowdchoir}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── Guards ──────────────────────────────────────────────────────────────────
if [[ "${EUID}" -ne 0 ]]; then
    echo "ERROR: run as root (sudo ./setup.sh)" >&2
    exit 1
fi

echo "==> WiFi interface : ${WIFI_IFACE}"
echo "==> AP address     : ${AP_IP}"
echo "==> Service user   : ${SERVICE_USER}"
echo "==> App directory  : ${APP_DIR}"
echo

# ── 1. Packages ─────────────────────────────────────────────────────────────
echo "==> Installing hostapd + dnsmasq ..."
apt-get update -y
apt-get install -y hostapd dnsmasq
systemctl unmask hostapd || true

# ── 2. Static AP address (dhcpcd) ───────────────────────────────────────────
echo "==> Configuring static address on ${WIFI_IFACE} ..."
DHCPCD=/etc/dhcpcd.conf
if [[ -f "${DHCPCD}" ]] && ! grep -q "# crowdchoir-ap" "${DHCPCD}"; then
    cat >>"${DHCPCD}" <<EOF

# crowdchoir-ap
interface ${WIFI_IFACE}
static ip_address=${AP_IP}/24
nohook wpa_supplicant
EOF
    echo "    added AP block to ${DHCPCD}"
else
    echo "    ${DHCPCD} already configured or missing (skipping)"
fi

# ── 3. hostapd ──────────────────────────────────────────────────────────────
echo "==> Installing hostapd.conf ..."
sed "s/^interface=.*/interface=${WIFI_IFACE}/" \
    "${SCRIPT_DIR}/hostapd.conf" >/etc/hostapd/hostapd.conf
if ! grep -q 'crowdchoir' /etc/default/hostapd 2>/dev/null; then
    echo 'DAEMON_CONF="/etc/hostapd/hostapd.conf"  # crowdchoir' \
        >>/etc/default/hostapd
fi

# ── 4. dnsmasq ──────────────────────────────────────────────────────────────
echo "==> Installing dnsmasq.conf (with DNS spoof) ..."
[[ -f /etc/dnsmasq.conf && ! -f /etc/dnsmasq.conf.orig ]] && \
    cp /etc/dnsmasq.conf /etc/dnsmasq.conf.orig
sed -e "s/^interface=.*/interface=${WIFI_IFACE}/" \
    -e "s#10\.0\.0\.1#${AP_IP}#g" \
    "${SCRIPT_DIR}/dnsmasq.conf" >/etc/dnsmasq.conf

# ── 5. systemd service ──────────────────────────────────────────────────────
echo "==> Installing crowdchoir.service ..."
sed -e "s#^User=.*#User=${SERVICE_USER}#" \
    -e "s#^WorkingDirectory=.*#WorkingDirectory=${APP_DIR}#" \
    -e "s#^ExecStart=.*#ExecStart=${APP_DIR}/target/release/crowdchoir-server#" \
    -e "s#10\.0\.0\.1#${AP_IP}#g" \
    "${SCRIPT_DIR}/crowdchoir.service" >/etc/systemd/system/crowdchoir.service

# ── 6. Build the server if needed ───────────────────────────────────────────
BIN="${APP_DIR}/target/release/crowdchoir-server"
if [[ ! -x "${BIN}" ]]; then
    echo "==> Release binary not found, building (this can take a while) ..."
    sudo -u "${SERVICE_USER}" bash -c "cd '${APP_DIR}' && cargo build --release"
else
    echo "==> Found existing binary: ${BIN}"
fi

# ── 7. Enable + start ───────────────────────────────────────────────────────
echo "==> Enabling and starting services ..."
systemctl daemon-reload
systemctl enable --now hostapd dnsmasq crowdchoir
systemctl restart hostapd dnsmasq crowdchoir

echo
echo "==> Done! Join the 'CrowdChoir' WiFi and the portal should pop up."
echo "    App URL: https://choir.party:5000  (or https://${AP_IP}:5000)"
echo
echo "    Check status:  systemctl status crowdchoir hostapd dnsmasq"
echo "    View logs:     journalctl -u crowdchoir -f"
