Absolutely — here is a **fully consolidated, structured, clean document** that covers *everything* from our entire conversation:

* CGNAT behavior
* Tunneling basics
* WireGuard architecture
* VPS setup
* Kernel and userspace clients
* Exit-node (internet through VPS)
* Routing, NAT, forwarding
* Adding peers
* Binding services to WireGuard IP
* Testing connectivity

This is a **complete reference**.

If you want, I can also generate this as a **PDF**, **Markdown**, or **DOCX** file.

---

# 📄 **Full Document: WireGuard, CGNAT Bypass, Multi-Peer Routing, Exit Node Setup, and Service Exposure**

---

# 1. **Understanding CGNAT**

## What CGNAT is

Carrier-Grade NAT (CGNAT) is an ISP-level NAT system where many customers share a public IP.
Key effects:

* Outbound connections **work**
* Inbound connections **do not work**
* Your device has a private ISP-side IP (100.x.x.x or 10.x.x.x)
* Peer-to-peer connections often fail unless tunneling is used

## Why torrents can seed under CGNAT

Torrent peers connect **outbound**, and seeding only requires outbound connections.
If a peer behind CGNAT cannot accept inbound, it can still upload to those who *can* receive inbound.

---

# 2. **Tunneling and How It Avoids CGNAT**

Tunneling creates an encrypted path:

```
Device (CGNAT) → outbound UDP → VPS (Public IP)
```

Once established:

* Both sides can exchange packets
* VPS can route traffic between multiple CGNAT devices
* CGNAT is bypassed because all traffic appears as outbound UDP

---

# 3. **WireGuard Overview**

WireGuard is extremely fast, simple, secure VPN software.
Two implementations exist:

### Kernel WireGuard

* Built into Linux kernel
* Fastest
* Recommended for VPS

### Userspace WireGuard

* `wireguard-go` (official)
* `wireguard-rs` (Rust)
* Required on macOS, Windows, some embedded hardware
* Requires a **TUN** interface

---

# 4. **TUN Interfaces**

A TUN device is a **virtual network interface** that transports raw IP packets between kernel and userspace.

Examples:

* `wg0`
* `tun0`
* `ts0` (Tailscale)

Userspace WireGuard reads/writes packets via TUN.

Kernel WireGuard does not need this and integrates directly with kernel networking.

---

# 5. **Why enable IP forwarding on the VPS**

VPS must act as a **router**:

```
Client A → wg0 → VPS → wg0 → Client B
```

Linux will NOT forward packets between interfaces unless:

```
sysctl -w net.ipv4.ip_forward=1
```

This is required for:

* Client A ↔ Client B communication
* Internet exit node (VPS as gateway)

---

# 6. **VPS WireGuard Server Setup**

### Install:

```
sudo apt update
sudo apt install wireguard
```

### Keys:

```
wg genkey | tee server_private.key | wg pubkey > server_public.key
```

### Example `/etc/wireguard/wg0.conf`:

```ini
[Interface]
Address = 10.0.0.1/24
ListenPort = 51820
PrivateKey = <SERVER_PRIVATE_KEY>

# Client A
[Peer]
PublicKey = <CLIENT_A_PUBLIC>
AllowedIPs = 10.0.0.2/32
```

Start service:

```
sudo wg-quick up wg0
sudo systemctl enable wg-quick@wg0
```

---

# 7. **VPS Routing Configuration**

### Enable forwarding:

```
sudo sysctl -w net.ipv4.ip_forward=1
echo "net.ipv4.ip_forward=1" | sudo tee -a /etc/sysctl.conf
```

### NAT (for exit node mode):

```
sudo iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
sudo iptables -A FORWARD -i wg0 -o eth0 -j ACCEPT
sudo iptables -A FORWARD -i eth0 -o wg0 -m state --state RELATED,ESTABLISHED -j ACCEPT
```

Persist:

```
sudo apt install iptables-persistent
```

---

# 8. **Client Setup (Kernel WireGuard on Linux)**

### Install:

```
sudo apt install wireguard
```

### Keys:

```
wg genkey | tee privatekey | wg pubkey > publickey
```

### Example `/etc/wireguard/wg0.conf`:

```ini
[Interface]
Address = 10.0.0.2/32
PrivateKey = <CLIENT_PRIVATE>
DNS = 1.1.1.1

[Peer]
PublicKey = <SERVER_PUBLIC>
Endpoint = <VPS_PUBLIC_IP>:51820
AllowedIPs = 10.0.0.0/24
PersistentKeepalive = 25
```

Start:

```
sudo wg-quick up wg0
```

---

# 9. **Client Setup on macOS (Userspace)**

WireGuard macOS always uses userspace (`wireguard-go`).

Steps:

* Install WireGuard.app
* Import config
* Activate tunnel

Config identical to Linux.

---

# 10. **Userspace WireGuard (wireguard-rs or wireguard-go)**

### Good for:

* macOS
* Windows
* routers
* embedded systems

Config file is the SAME as kernel WireGuard.

To run:

```
sudo ./wireguard-rs -f /etc/wireguard/wg0.conf
```

or:

```
sudo wg-quick up wg0
```

---

# 11. **Using VPS as full exit node (client internet routing)**

On **client**:

```
AllowedIPs = 0.0.0.0/0
DNS = 1.1.1.1
```

Then restart:

```
sudo wg-quick down wg0
sudo wg-quick up wg0
```

Client routing should now show:

```
default dev wg0
```

Test:

```
curl ifconfig.me
```

Should display VPS public IP.

---

# 12. **Exposing services over WireGuard (safe, private)**

Bind services to WG IP (e.g., `10.0.0.2`):

### SSH

`/etc/ssh/sshd_config`:

```
ListenAddress 10.0.0.2
```

### HTTP (Nginx)

```
listen 10.0.0.2:80;
```

### FTP (vsftpd)

```
listen_address=10.0.0.2
pasv_address=10.0.0.2
```

This restricts access to VPN-only.

---

# 13. **Testing connectivity**

Run on Client B:

```bash
curl http://10.0.0.2:8080
ping 10.0.0.1
ping 10.0.0.3
```

Run on VPS:

```bash
sudo tcpdump -n -i wg0
sudo tcpdump -n -i eth0 udp port 51820
```

---

# 14. **Adding another WireGuard peer**

On VPS:

```ini
# Client B
[Peer]
PublicKey = <CLIENT_B_PUBLIC>
AllowedIPs = 10.0.0.3/32
```

Client B config:

```ini
[Interface]
Address = 10.0.0.3/32
PrivateKey = <CLIENT_B_PRIVATE>
DNS = 1.1.1.1

[Peer]
PublicKey = <SERVER_PUBLIC>
Endpoint = <VPS_PUBLIC_IP>:51820
AllowedIPs = 0.0.0.0/0
PersistentKeepalive = 25
```

Restart:

```
sudo wg setconf wg0 /etc/wireguard/wg0.conf
```

---

# 15. **Troubleshooting Summary**

### If VPN connects but no internet:

* Wrong `AllowedIPs`
* Missing NAT on VPS
* Missing forwarding on VPS
* Old default route still active on client
* DNS not set

### If client cannot reach other peers:

* Wrong AllowedIPs on VPS
* Missing peer entries
* Duplicate IP assignments
* Missing forwarding rules

### If no handshake:

* Endpoint unreachable
* Firewall blocking UDP 51820
* Keys mismatched

---

# 16. **Summary of Full Architecture**

```
           ┌──────────────────────┐
           │      Client A        │
           │   (CGNAT, 10.0.0.2)  │
           └───────▲───┬──────────┘
                   │   │
                   │   │ Encrypted WireGuard
                   │   │ UDP Tunnel
                   ▼   ▼
         ┌────────────────────────────┐
         │            VPS             │
         │ Public IP, wg0 = 10.0.0.1  │
         │ IP forwarding + NAT        │
         └───────────▲────┬──────────┘
                     │    │
                     │    │
                     ▼    ▼
           ┌──────────────────────┐
           │      Client B        │
           │   (CGNAT, 10.0.0.3)  │
           └──────────────────────┘
```

Clients behind CGNAT can communicate with each other via VPS and can route internet through VPS.

---

# 17. **Optional Enhancements**

* Headscale or wg-easy dashboard
* Policy-based routing (split tunnel)
* IPv6 support
* Failover VPS
* Monitoring (Prometheus exporter)

---

# 18. **Conclusion**

You now have a complete system that:

* Bypasses CGNAT
* Connects multiple private devices securely
* Acts as an internet exit node
* Routes traffic between clients
* Lets you expose services safely
* Works with kernel or userspace implementations
* Easily scales to many peers

---

If you want this **converted to a PDF, DOCX, or Markdown file**, tell me the format and I'll generate it.

