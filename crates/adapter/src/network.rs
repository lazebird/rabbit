//! Network interface utilities

use schema::NetworkProvider;
use std::collections::HashMap;
use std::net::Ipv4Addr;

/// Default implementation of [`schema::NetworkProvider`].
pub struct DefaultNetworkProvider;

impl NetworkProvider for DefaultNetworkProvider {
    fn get_mac_from_arp(&self, ip: Ipv4Addr) -> Option<String> {
        get_mac_from_arp(ip)
    }

    fn get_local_mac_table(&self) -> HashMap<Ipv4Addr, String> {
        #[cfg(target_os = "linux")]
        return get_local_ip_mac_table();
        #[cfg(target_os = "windows")]
        return get_local_ip_mac_table();
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        HashMap::new()
    }
}

/// Get MAC address from kernel ARP cache.
///
/// - Linux: `SIOCGARP` ioctl with correct interface name (from `getifaddrs`)
/// - Windows: `arp -a` cache lookup
///
/// Returns `None` if the MAC could not be resolved.
pub fn get_mac_from_arp(ip: Ipv4Addr) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        get_mac_via_ioctl(ip)
    }
    #[cfg(target_os = "windows")]
    {
        get_mac_from_arp_cache(ip)
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    None
}

/// Windows ARP lookup via [`GetIpNetTable2`] with UDP-triggered ARP.
///
/// `surge_ping` on Windows uses `IcmpSendEcho`, which does NOT update
/// the kernel ARP cache.  We therefore send a 1-byte UDP packet to
/// force the IP stack to resolve the target MAC via ARP before
/// querying the cache via [`GetIpNetTable2`].
#[cfg(target_os = "windows")]
fn get_mac_from_arp_cache(ip: Ipv4Addr) -> Option<String> {
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        let addr = std::net::SocketAddr::V4(std::net::SocketAddrV4::new(ip, 9));
        if socket.connect(addr).is_ok() {
            let _ = socket.send(&[0u8; 1]);
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(20));

    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetIpNetTable2, FreeMibTable, MIB_IPNET_TABLE2,
    };
    use windows_sys::Win32::Networking::WinSock::AF_INET;
    use windows_sys::Win32::Foundation::NO_ERROR;

    let mut table_ptr: *mut MIB_IPNET_TABLE2 = std::ptr::null_mut();
    let status = unsafe { GetIpNetTable2(AF_INET, &mut table_ptr) };

    if status != NO_ERROR || table_ptr.is_null() {
        return None;
    }

    struct FreeGuard(*mut MIB_IPNET_TABLE2);
    impl Drop for FreeGuard {
        fn drop(&mut self) {
            unsafe { FreeMibTable(self.0 as *mut _) };
        }
    }
    let _guard = FreeGuard(table_ptr);

    let table = unsafe { &*table_ptr };
    let num_entries = table.NumEntries as usize;

    if num_entries == 0 {
        return None;
    }

    let entries = unsafe {
        std::slice::from_raw_parts(
            &table.Table as *const _ as *const windows_sys::Win32::NetworkManagement::IpHelper::MIB_IPNET_ROW2,
            num_entries,
        )
    };

    let target_u32 = u32::from(ip);

    for entry in entries.iter() {
        // S_addr in GetIpNetTable2 entries has inconsistent byte
        // ordering; check both the raw value and its byte swap.
        let s_addr = unsafe { entry.Address.Ipv4.sin_addr.S_un.S_addr };
        if (s_addr == target_u32 || s_addr == target_u32.swap_bytes())
            && entry.PhysicalAddressLength >= 6
        {
            let mac = &entry.PhysicalAddress;
            if mac[..6].iter().any(|&b| b != 0) {
                return Some(format!(
                    "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5],
                ));
            }
        }
    }

    None
}

/// Windows: enumerate local interface IP→MAC mappings via
/// [`GetAdaptersAddresses`].
///
/// This is used at scan start to resolve the scanner's own IP(s)
/// without querying the kernel ARP cache (which has no self-entry).
#[cfg(target_os = "windows")]
fn get_local_ip_mac_table() -> HashMap<Ipv4Addr, String> {
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH,
        GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_MULTICAST, GAA_FLAG_SKIP_DNS_SERVER,
    };
    use windows_sys::Win32::Networking::WinSock::AF_INET;
    use windows_sys::Win32::Foundation::NO_ERROR;

    let mut table = HashMap::new();

    let flags = GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER;
    let mut buf_len: u32 = 0;

    // SAFETY: GetAdaptersAddresses writes the required buffer length
    // into buf_len and returns ERROR_BUFFER_OVERFLOW (111).
    let ret = unsafe {
        GetAdaptersAddresses(
            AF_INET as u32,
            flags,
            std::ptr::null(),
            std::ptr::null_mut(),
            &mut buf_len,
        )
    };
    if ret == NO_ERROR || buf_len == 0 {
        return table;
    }

    let mut buf = vec![0u8; buf_len as usize];
    let adapters = buf.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

    if unsafe {
        GetAdaptersAddresses(
            AF_INET as u32,
            flags,
            std::ptr::null(),
            adapters,
            &mut buf_len,
        )
    } != NO_ERROR
    {
        return table;
    }

    let mut cur = adapters;
    while !cur.is_null() {
        let a = unsafe { &*cur };

        if a.PhysicalAddressLength >= 6 {
            let mac_bytes = &a.PhysicalAddress[..6];
            if mac_bytes.iter().any(|&b| b != 0) {
                let mac = format!(
                    "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    mac_bytes[0], mac_bytes[1], mac_bytes[2],
                    mac_bytes[3], mac_bytes[4], mac_bytes[5],
                );

                let mut uni = a.FirstUnicastAddress;
                while !uni.is_null() {
                    let u = unsafe { &*uni };
                    if !u.Address.lpSockaddr.is_null() {
                        let sin = unsafe {
                            &*(u.Address.lpSockaddr as *const windows_sys::Win32::Networking::WinSock::SOCKADDR_IN)
                        };
                        if sin.sin_family == AF_INET {
                            // S_addr from GetAdaptersAddresses stores IP
                            // octets in host byte order (LE on x86), so
                            // to_le_bytes() gives the octets in order.
                            let s_addr = unsafe { sin.sin_addr.S_un.S_addr };
                            let ip = Ipv4Addr::from(s_addr.to_le_bytes());
                            table.insert(ip, mac.clone());
                        }
                    }
                    uni = u.Next;
                }
            }
        }

        cur = a.Next;
    }

    table
}

// ─── Linux: SIOCGARP ioctl ─────────────────────────────────────────

/// Linux: query kernel ARP table via `SIOCGARP` ioctl.
///
/// **Root cause of previous failures**: the kernel requires `arp_dev` to
/// identify the network device.  With an empty `arp_dev`, `dev` stays
/// NULL and [`neigh_lookup(&arp_tbl, &ip, NULL)`][kernel] cannot find
/// any entry because neighbour entries are indexed per-device.
///
/// The fix:
/// 1. Enumerate interfaces via `getifaddrs()`
/// 2. Find the interface whose subnet contains `ip`
/// 3. Set `arp_dev` to that interface name before calling `SIOCGARP`
///
/// [kernel]: https://github.com/torvalds/linux/blob/v6.6/net/ipv4/arp.c#L1118-L1138
#[cfg(target_os = "linux")]
fn get_mac_via_ioctl(ip: Ipv4Addr) -> Option<String> {
    use std::mem;

    const SIOCGARP: libc::c_ulong = 0x8954;
    const ATF_COM: libc::c_int = 0x02;
    const IFNAMSIZ: usize = 16;

    let if_name = find_interface_for_ip(ip)?;

    // ── Build `struct arpreq` ──────────────────────────────────────
    //
    // Layout (matches C `struct arpreq` from `<linux/if_arp.h>`):
    //
    //   ┌──────────────────────────────┬────────┬──────┐
    //   │  field                       │ offset │ size │
    //   ├──────────────────────────────┼────────┼──────┤
    //   │ arp_pa (sockaddr)            │   0    │  16  │
    //   │  ├ sa_family (u16)           │   0    │   2  │
    //   │  ├ sin_port (padding)        │   2    │   2  │
    //   │  └ sin_addr.s_addr (u32)     │   4    │   4  │
    //   │ arp_ha (sockaddr)            │  16    │  16  │
    //   │  ├ sa_family                 │  16    │   2  │
    //   │  └ sa_data (MAC)             │  18    │   6  │
    //   │ arp_flags (c_int)            │  32    │   4  │
    //   │ arp_netmask (sockaddr)       │  36    │  16  │
    //   │ arp_dev (char[16])           │  52    │  16  │
    //   └──────────────────────────────┴────────┴──────┘
    #[repr(C)]
    struct arpreq {
        arp_pa_family: u16,
        arp_pa_data: [u8; 14],
        arp_ha_family: u16,
        arp_ha_data: [u8; 14],
        arp_flags: libc::c_int,
        arp_netmask_family: u16,
        arp_netmask_data: [u8; 14],
        arp_dev: [libc::c_char; IFNAMSIZ],
    }

    let octets = ip.octets();

    unsafe {
        let sock = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0);
        if sock < 0 {
            return None;
        }

        let mut req: arpreq = mem::zeroed();
        req.arp_pa_family = libc::AF_INET as u16;
        req.arp_pa_data[2] = octets[0];
        req.arp_pa_data[3] = octets[1];
        req.arp_pa_data[4] = octets[2];
        req.arp_pa_data[5] = octets[3];

        let name_bytes = if_name.as_bytes();
        let len = name_bytes.len().min(IFNAMSIZ - 1);
        // SAFETY: arp_dev is a fixed-size C char array.
        let dev_slice = std::slice::from_raw_parts_mut(
            req.arp_dev.as_mut_ptr() as *mut u8,
            IFNAMSIZ,
        );
        dev_slice[..len].copy_from_slice(&name_bytes[..len]);
        dev_slice[len] = 0;

        let ret = libc::ioctl(sock, SIOCGARP, &mut req as *mut _);
        libc::close(sock);

        if ret == 0 && (req.arp_flags & ATF_COM) != 0 {
            let mac = &req.arp_ha_data[..6];
            if mac.iter().any(|&b| b != 0) {
                return Some(format!(
                    "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5],
                ));
            }
        }
    }
    None
}

/// Find the non-loopback network interface whose subnet contains `ip`.
///
/// Uses `getifaddrs()` to enumerate all AF_INET interfaces and checks
/// `(interface_ip & netmask) == (target_ip & netmask)`.
#[cfg(target_os = "linux")]
fn find_interface_for_ip(ip: Ipv4Addr) -> Option<String> {
    let mut addrs: *mut libc::ifaddrs = std::ptr::null_mut();

    let ip_nbo = u32::from(ip);

    if unsafe { libc::getifaddrs(&mut addrs) } != 0 || addrs.is_null() {
        return None;
    }

    let mut result: Option<String> = None;
    let mut walker = addrs;

    while !walker.is_null() {
        let ifa = unsafe { &*walker };

        if !ifa.ifa_addr.is_null()
            && !ifa.ifa_netmask.is_null()
            && unsafe { (*ifa.ifa_addr).sa_family } == libc::AF_INET as libc::sa_family_t
        {
            let addr = unsafe { &*(ifa.ifa_addr as *const libc::sockaddr_in) };
            let mask = unsafe { &*(ifa.ifa_netmask as *const libc::sockaddr_in) };

            let ifa_ip_nbo = u32::from_be(addr.sin_addr.s_addr);
            let ifa_mask_nbo = u32::from_be(mask.sin_addr.s_addr);

            if (ifa_ip_nbo & ifa_mask_nbo) == (ip_nbo & ifa_mask_nbo) {
                let name = unsafe { std::ffi::CStr::from_ptr(ifa.ifa_name) }
                    .to_string_lossy()
                    .into_owned();

                if name != "lo" {
                    result = Some(name);
                    break;
                }
            }
        }

        walker = unsafe { (*walker).ifa_next };
    }

    unsafe { libc::freeifaddrs(addrs) };
    result
}

/// Build a table of IP → MAC for all local non-loopback interfaces.
///
/// Enumerates interfaces via `getifaddrs()` and reads the MAC from
/// `/sys/class/net/<name>/address`.
#[cfg(target_os = "linux")]
fn get_local_ip_mac_table() -> HashMap<Ipv4Addr, String> {
    let mut table = HashMap::new();
    let mut addrs: *mut libc::ifaddrs = std::ptr::null_mut();

    if unsafe { libc::getifaddrs(&mut addrs) } != 0 || addrs.is_null() {
        return table;
    }

    let mut walker = addrs;
    while !walker.is_null() {
        let ifa = unsafe { &*walker };
        if !ifa.ifa_addr.is_null()
            && unsafe { (*ifa.ifa_addr).sa_family } == libc::AF_INET as libc::sa_family_t
        {
            let name = unsafe { std::ffi::CStr::from_ptr(ifa.ifa_name) }
                .to_string_lossy()
                .into_owned();

            if name != "lo" {
                let addr = unsafe { &*(ifa.ifa_addr as *const libc::sockaddr_in) };
                let ip = Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr));

                let mac = std::fs::read_to_string(format!("/sys/class/net/{}/address", name))
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|m| !m.is_empty() && m != "00:00:00:00:00:00");

                if let Some(m) = mac {
                    table.insert(ip, m);
                }
            }
        }
        walker = unsafe { (*walker).ifa_next };
    }

    unsafe { libc::freeifaddrs(addrs) };
    table
}

#[cfg(test)]
#[cfg(target_os = "windows")]
mod windows_arp_tests {
    use super::*;

    /// Verify that known ARP table entries from the LAN are readable
    /// via `get_mac_from_arp`.  These are the entries the scanner
    /// needs to resolve — if this test fails, the UDP probe +
    /// `GetIpNetTable2` path is broken.
    ///
    /// MACs are in `xx:xx:xx:xx:xx:xx` format (our code), while
    /// `arp -a` uses `xx-xx-xx-xx-xx-xx`.
    #[test]
    fn test_arp_read_gateway() {
        let mac = get_mac_from_arp(Ipv4Addr::new(192, 168, 31, 1));
        assert_eq!(mac.as_deref(), Some("64:64:4a:28:c2:d0"));
    }

    /// Verify that the local interface MAC table returns at least one
    /// entry for the host's own IP (which `get_mac_from_arp` cannot
    /// resolve because the kernel ARP cache has no self-entry).
    #[test]
    fn test_local_mac_table_contains_self() {
        let table = get_local_ip_mac_table();
        assert!(
            !table.is_empty(),
            "get_local_ip_mac_table returned empty — no local IPv4 interfaces with valid MAC found"
        );
    }

    #[test]
    fn test_arp_read_multiple_hosts() {
        let cases = [
            (192, 168, 31, 1,   "64:64:4a:28:c2:d0"),
            (192, 168, 31, 49,  "50:ec:50:41:99:51"),
            (192, 168, 31, 136, "12:ce:7c:bd:fa:ac"),
            (192, 168, 31, 179, "e4:24:6c:71:c2:cc"),
            (192, 168, 31, 198, "04:cf:8c:69:52:7b"),
        ];
        for (a, b, c, d, expected) in cases {
            let ip = Ipv4Addr::new(a, b, c, d);
            let mac = get_mac_from_arp(ip);
            assert_eq!(
                mac.as_deref(),
                Some(expected),
                "Failed to resolve MAC for {}.{}.{}.{} (expected {})",
                a, b, c, d, expected,
            );
        }
    }
}


