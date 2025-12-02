# Breaking cgnat through wireguard (AI Generated)

Here is a clean, structured **document** outlining all of the key findings from our conversation so far.
It summarizes CGNAT behavior, tunneling concepts, WireGuard networking, Tailscale/DERP, relays, TURN,
 VPN routing, and how to expose services (FTP/SSH/HTTP) through WireGuard.

If you'd like this as a PDF, DOCX, or Markdown file, I can generate that too.

---

# 📄 **Document: Networking Behind CGNAT, Tunneling, WireGuard, Routing, and Service Binding**

---

## **1. Understanding CGNAT and Its Implications**

### **What is CGNAT?**

Carrier-Grade NAT (CGNAT) is a NAT layer operated by the ISP. It prevents devices from receiving inbound
 connections because many customers share a single public IP.

### **Key Properties**

* ✔ Outbound connections **always** work
* ❌ Inbound connections **never** reach your device
* ✔ Torrents can **seed** because upload happens over outbound connections
* ❌ Two CGNAT peers generally **cannot connect directly**

---

## **2. How Tunneling Works**

Tunneling creates a private, encrypted link between two endpoints.
A CGNAT device initiates an outbound connection to a public server.

```
Device (CGNAT) → outbound → VPS (public IP)
```

Once the tunnel exists, the VPS becomes a reachable point for:

* Routing
* NAT traversal
* Private networking

---

## **3. WireGuard as a High-Performance Tunnel**

### **Why WireGuard is fast**

* Kernel-level (on Linux)
* Uses minimal overhead UDP
* Uses modern crypto (ChaCha20)
* Handles Gbps throughput

### **Where WireGuard runs**

* Kernel implementation (fastest)
* Userspace implementations:

  * `wireguard-go`
  * `wireguard-rs`

Userspace implementations require a **TUN** device.

---

## **4. What is a TUN Device?**

A **TUN interface** is a virtual network card used by VPNs.
It transports **IP packets** between kernel and userspace programs.

Examples:

* `wg0`
* `tun0`
* `ts0` (Tailscale)

VPN software reads/writes packets through this interface.

---

## **5. Building a CGNAT-Proof Network Using WireGuard**

### **Topology**

```
Device A (CGNAT) → WireGuard → VPS → WireGuard → Device B (CGNAT)
```

### **Process**

1. Each device initiates an outbound tunnel to the VPS.
2. VPS routes traffic between peers.
3. Devices communicate through WireGuard private IPs (e.g., 10.0.0.2 ↔ 10.0.0.3).

### **Result**

* Devices behind CGNAT can reach each other
* Any TCP/UDP service becomes reachable over the VPN

---

## **6. VPS Routing Requirements**

On VPS:

* Enable IP forwarding:

  ```
  sysctl -w net.ipv4.ip_forward=1
  ```
* Add peers in `wg0.conf`
* Route `10.0.0.0/24` between clients
* (Optional) Enable NAT if clients access the internet

---

## **7. Running WireGuard-rs on CGNAT Devices**

* Fully compatible with kernel WireGuard
* Requires TUN interface
* Slightly slower: userspace overhead
* Perfect for devices where WG kernel module is unavailable

### **Yes — You can run:**

* `wireguard-rs` on Device A and B
* Kernel WireGuard on VPS

This combination works seamlessly.

---

## **8. Understanding Relays: DERP, TURN, and When They Are Needed**

### **DERP (Tailscale)**

* Tailscale-specific relay
* Used only when P2P fails
* Not a VPN, just a TCP/UDP relay for encrypted packets

### **TURN**

* Generic WebRTC relay (RFC 5766/8656)
* Intended for media
* Slow for bulk traffic (5–30 Mbps)

### **WireGuard**

* No relays
* Direct UDP
* Requires one endpoint to be reachable (e.g., VPS)

---

## **9. Routing Traffic: Kernel vs Relay**

### **Kernel routing**

* VPS forwards packets at high speed
* Equivalent to a router
* Used in WireGuard-based networks
* Perfect for multi-peer connections behind CGNAT

### **Relay servers (TURN/DERP)**

* Application-level
* Must handle every packet
* Slower than kernel routing
* Only used when direct paths fail

---

## **10. Binding Services to WireGuard IPs**

To expose services *only* through the VPN (10.0.0.x), bind them to the WireGuard IP.

### **Bind SSH**

`/etc/ssh/sshd_config`:

```
ListenAddress 10.0.0.2
ListenAddress 127.0.0.1
```

### **Bind HTTP (Nginx)**

```
listen 10.0.0.2:80;
```

### **Bind FTP (vsftpd)**

```
listen_address=10.0.0.2
pasv_address=10.0.0.2
```

### Why do this?

* Exposes services **only** to WireGuard peers
* Safer than exposing to LAN or public internet
* Works behind CGNAT because traffic is inside the VPN

---

## **11. Accessing a Service Behind CGNAT**

With the VPN established:

```
Device B → 10.0.0.2:21   (FTP)
Device B → 10.0.0.2:22   (SSH)
Device B → http://10.0.0.2  (Web)
```

CGNAT does not matter because:

* Both devices reach the VPS
* The VPS forwards using kernel routing
* WireGuard encrypts all traffic end-to-end

---

## **12. Summary of Key Points**

* CGNAT blocks inbound connections.
* Outbound tunnels (WireGuard) bypass CGNAT entirely.
* A VPS acts as a **router**, not a relay.
* WireGuard-rs works perfectly with kernel WireGuard.
* TUN interfaces allow userspace VPNs to function.
* DERP/TURN relay systems are slow; WireGuard routing is fast.
* Binding services to the WG IP restricts access to the private VPN only.
* Devices behind CGNAT can communicate with each other securely via the VPS.

---

## **13. Optional Next Steps**

I can generate:

✔ A PDF of this document
✔ A full WireGuard configuration bundle
✔ A diagram (visual network layout)
✔ A Headscale-based self-hosted Tailscale alternative
✔ Auto-start scripts (systemd)
✔ Secure SFTP-only configuration (recommended over FTP)

---

If you'd like this turned into a downloadable file (PDF, DOCX, MD) just tell me which format!

