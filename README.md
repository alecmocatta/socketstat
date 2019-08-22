# socketstat

[![Crates.io](https://img.shields.io/crates/v/socketstat.svg?maxAge=86400)](https://crates.io/crates/socketstat)
[![MIT / Apache 2.0 licensed](https://img.shields.io/crates/l/socketstat.svg?maxAge=2592000)](#License)
[![Build Status](https://dev.azure.com/alecmocatta/socketstat/_apis/build/status/tests?branchName=master)](https://dev.azure.com/alecmocatta/socketstat/_build/latest?branchName=master)

[Docs](https://docs.rs/socketstat/0.1.0)

Get socket information and statistics.

Currently works on macOS only, PRs for other platforms welcome!

## Example

```rust
#[cfg(unix)]
use std::os::unix::io::AsRawFd;
#[cfg(windows)]
use std::os::windows::io::AsRawSocket;
use socketstat::socketstat;

let sock = std::net::TcpStream::connect("google.com:80").unwrap();

#[cfg(unix)]
let fd = sock.as_raw_fd();
#[cfg(windows)]
let fd = sock.as_raw_socket();

println!("{:#?}", socketstat(fd));

// prints:
//   Ok(
//       SocketStat {
//           unreceived: 0,
//           unsent: 0,
//           connection_info: tcp_connection_info {
//               tcpi_state: "ESTABLISHED",
//               tcpi_snd_wscale: 8,
//               tcpi_rcv_wscale: 6,
//               tcpi_options: 7,
//               tcpi_flags: 0,
//               tcpi_rto: 0,
//               tcpi_maxseg: 1368,
//               tcpi_snd_ssthresh: 1073725440,
//               tcpi_snd_cwnd: 4380,
//               tcpi_snd_wnd: 60192,
//               tcpi_snd_sbbytes: 0,
//               tcpi_rcv_wnd: 131328,
//               tcpi_rttcur: 79,
//               tcpi_srtt: 79,
//               tcpi_rttvar: 39,
//               tcpi_tfo: 0,
//               tcpi_txpackets: 0,
//               tcpi_txbytes: 0,
//               tcpi_txretransmitbytes: 0,
//               tcpi_rxpackets: 0,
//               tcpi_rxbytes: 0,
//               tcpi_rxoutoforderbytes: 0,
//               tcpi_txretransmitpackets: 0,
//           },
//           socket_info: tcp_sockinfo {
//               tcpsi_ini: in_sockinfo {
//                   insi_fport: 80,
//                   insi_lport: 52621,
//                   insi_gencnt: 100950561,
//                   insi_flags: 8390720,
//                   insi_flow: 0,
//                   insi_vflag: "IPV4",
//                   insi_ip_ttl: 64,
//                   rfu_1: 0,
//               },
//               tcpsi_state: "ESTABLISHED",
//               tcpsi_timer: [
//                   0,
//                   0,
//                   7200079,
//                   0,
//               ],
//               tcpsi_mss: 1368,
//               tcpsi_flags: 1140851680,
//               rfu_1: 0,
//               tcpsi_tp: 9662996336038732135,
//           },
//       },
//   )
```

## Note

On macOS this calls:
* `getsockopt(fd, IPPROTO_TCP, TCP_CONNECTION_INFO, ...)`
* `proc_pidfdinfo(getpid(), fd, PROC_PIDFDSOCKETINFO, ...)`
* `ioctl(fd, FIONREAD, ...)`
* `getsockopt(fd, SOL_SOCKET, SO_NWRITE, ...)`

Other sources to explore:
* `sysctl([CTL_NET, PF_INET, IPPROTO_TCP, TCPCTL_PCBLIST], ...`
* <https://stackoverflow.com/questions/31263289/on-linux-mac-windows-is-it-possible-to-access-the-tcp-timestamp-and-or-rtt-in-u>

## License
Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE.txt](LICENSE-APACHE.txt) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT.txt](LICENSE-MIT.txt) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
