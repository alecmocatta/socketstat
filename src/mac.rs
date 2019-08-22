#![allow(non_camel_case_types, trivial_casts, missing_copy_implementations, clippy::shadow_unrelated)]

use nix::libc;
use std::{
	convert::{TryFrom, TryInto}, fmt, mem
};

#[derive(Clone, Debug)]
pub struct SocketStat {
	unreceived: usize,
	unsent: usize,
	connection_info: tcp_connection_info,
	socket_info: tcp_sockinfo,
}

#[derive(Clone, Debug)]
pub struct Error(nix::Error);

pub fn socketstat(fd: int) -> Result<SocketStat, Error> {
	let unreceived = palaver::socket::unreceived(fd);
	let unsent = palaver::socket::unsent(fd);

	let mut connection_info: tcp_connection_info = unsafe { mem::zeroed() };
	let mut len: libc::socklen_t = std::mem::size_of::<tcp_connection_info>()
		.try_into()
		.unwrap();
	let res = unsafe {
		libc::getsockopt(
			fd,
			libc::IPPROTO_TCP,
			TCP_CONNECTION_INFO,
			&mut connection_info as *mut _ as *mut _,
			&mut len,
		)
	};
	let res = nix::errno::Errno::result(res).map_err(Error)?;
	assert_eq!(res, 0);

	let mut socket_info: socket_fdinfo = unsafe { std::mem::zeroed() };
	let len = unsafe {
		proc_pidfdinfo(
			libc::getpid(),
			fd,
			PROC_PIDFDSOCKETINFO,
			&mut socket_info as *mut _ as *mut _,
			std::mem::size_of::<socket_fdinfo>().try_into().unwrap(),
		)
	};
	#[allow(clippy::cast_sign_loss)]
	let socket_info = if len >= 0
		&& len as usize == std::mem::size_of::<socket_fdinfo>()
		&& socket_info.psi.soi_family == libc::AF_INET
		&& socket_info.psi.soi_kind == SOCKINFO_TCP
	{
		Ok(unsafe { socket_info.psi.soi_proto.pri_tcp })
	} else {
		Err(Error(nix::Error::from_errno(
			nix::errno::Errno::UnknownErrno,
		)))
	}?;

	Ok(SocketStat {
		unreceived,
		unsent,
		connection_info,
		socket_info,
	})
}

// It seems the two states aren't guaranteed to be the same:
// https://bugzilla.kernel.org/show_bug.cgi?id=33902

// https://github.com/apple/darwin-xnu/blob/a449c6a3b8014d9406c2ddbdc81795da24aa7443/bsd/netinet/tcp.h

const TCP_CONNECTION_INFO: libc::c_int = 0x106; /* State of TCP connection */

#[derive(Copy, Clone)]
#[repr(C)]
struct tcp_connection_info {
	tcpi_state: u8,      /* connection state */
	tcpi_snd_wscale: u8, /* Window scale for send window */
	tcpi_rcv_wscale: u8, /* Window scale for receive window */
	__pad1: u8,
	tcpi_options: u32, /* TCP options supported */
	// #define TCPCI_OPT_TIMESTAMPS    0x00000001 /* Timestamps enabled */
	// #define TCPCI_OPT_SACK          0x00000002 /* SACK enabled */
	// #define TCPCI_OPT_WSCALE        0x00000004 /* Window scaling enabled */
	// #define TCPCI_OPT_ECN           0x00000008 /* ECN enabled */
	tcpi_flags: u32, /* flags */
	// #define TCPCI_FLAG_LOSSRECOVERY 0x00000001
	// #define TCPCI_FLAG_REORDERING_DETECTED  0x00000002
	tcpi_rto: u32,          /* retransmit timeout in ms */
	tcpi_maxseg: u32,       /* maximum segment size supported */
	tcpi_snd_ssthresh: u32, /* slow start threshold in bytes */
	tcpi_snd_cwnd: u32,     /* send congestion window in bytes */
	tcpi_snd_wnd: u32,      /* send widnow in bytes */
	tcpi_snd_sbbytes: u32,  /* bytes in send socket buffer, including in-flight data */
	tcpi_rcv_wnd: u32,      /* receive window in bytes*/
	tcpi_rttcur: u32,       /* most recent RTT in ms */
	tcpi_srtt: u32,         /* average RTT in ms */
	tcpi_rttvar: u32,       /* RTT variance */
	tcpi_tfo: u32,
	// tcpi_tfo_cookie_req:1,             /* Cookie requested? */
	// tcpi_tfo_cookie_rcv:1,             /* Cookie received? */
	// tcpi_tfo_syn_loss:1,               /* Fallback to reg. TCP after SYN-loss */
	// tcpi_tfo_syn_data_sent:1,             /* SYN+data has been sent out */
	// tcpi_tfo_syn_data_acked:1,             /* SYN+data has been fully acknowledged */
	// tcpi_tfo_syn_data_rcv:1,             /* Server received SYN+data with a valid cookie */
	// tcpi_tfo_cookie_req_rcv:1,             /* Server received cookie-request */
	// tcpi_tfo_cookie_sent:1,             /* Server announced cookie */
	// tcpi_tfo_cookie_invalid:1,             /* Server received an invalid cookie */
	// tcpi_tfo_cookie_wrong:1,             /* Our sent cookie was wrong */
	// tcpi_tfo_no_cookie_rcv:1,             /* We did not receive a cookie upon our request */
	// tcpi_tfo_heuristics_disable:1,             /* TFO-heuristics disabled it */
	// tcpi_tfo_send_blackhole:1,             /* A sending-blackhole got detected */
	// tcpi_tfo_recv_blackhole:1,             /* A receiver-blackhole got detected */
	// tcpi_tfo_onebyte_proxy:1,             /* A proxy acknowledges all but one byte of the SYN */
	// __pad2:17,
	tcpi_txpackets: u64,
	tcpi_txbytes: u64,
	tcpi_txretransmitbytes: u64,
	tcpi_rxpackets: u64,
	tcpi_rxbytes: u64,
	tcpi_rxoutoforderbytes: u64,
	tcpi_txretransmitpackets: u64,
}
impl fmt::Debug for tcp_connection_info {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		let Self {
			tcpi_state,
			tcpi_snd_wscale,
			tcpi_rcv_wscale,
			__pad1,
			tcpi_options,
			tcpi_flags,
			tcpi_rto,
			tcpi_maxseg,
			tcpi_snd_ssthresh,
			tcpi_snd_cwnd,
			tcpi_snd_wnd,
			tcpi_snd_sbbytes,
			tcpi_rcv_wnd,
			tcpi_rttcur,
			tcpi_srtt,
			tcpi_rttvar,
			tcpi_tfo,
			tcpi_txpackets,
			tcpi_txbytes,
			tcpi_txretransmitbytes,
			tcpi_rxpackets,
			tcpi_rxbytes,
			tcpi_rxoutoforderbytes,
			tcpi_txretransmitpackets,
		} = self;
		fmt.debug_struct("tcp_connection_info")
			.field(
				"tcpi_state",
				&state_to_name(*tcpi_state).unwrap_or("<unknown>"),
			)
			.field("tcpi_snd_wscale", tcpi_snd_wscale)
			.field("tcpi_rcv_wscale", tcpi_rcv_wscale)
			.field("tcpi_options", tcpi_options)
			.field("tcpi_flags", tcpi_flags)
			.field("tcpi_rto", tcpi_rto)
			.field("tcpi_maxseg", tcpi_maxseg)
			.field("tcpi_snd_ssthresh", tcpi_snd_ssthresh)
			.field("tcpi_snd_cwnd", tcpi_snd_cwnd)
			.field("tcpi_snd_wnd", tcpi_snd_wnd)
			.field("tcpi_snd_sbbytes", tcpi_snd_sbbytes)
			.field("tcpi_rcv_wnd", tcpi_rcv_wnd)
			.field("tcpi_rttcur", tcpi_rttcur)
			.field("tcpi_srtt", tcpi_srtt)
			.field("tcpi_rttvar", tcpi_rttvar)
			.field("tcpi_tfo", tcpi_tfo)
			.field("tcpi_txpackets", tcpi_txpackets)
			.field("tcpi_txbytes", tcpi_txbytes)
			.field("tcpi_txretransmitbytes", tcpi_txretransmitbytes)
			.field("tcpi_rxpackets", tcpi_rxpackets)
			.field("tcpi_rxbytes", tcpi_rxbytes)
			.field("tcpi_rxoutoforderbytes", tcpi_rxoutoforderbytes)
			.field("tcpi_txretransmitpackets", tcpi_txretransmitpackets)
			.finish()
	}
}

// https://github.com/apple/darwin-xnu/blob/a449c6a3b8014d9406c2ddbdc81795da24aa7443/bsd/sys/proc_info.h

type int = libc::c_int;
type short = libc::c_short;
type off_t = libc::off_t;
type u_short = libc::c_ushort;
type u_char = libc::c_uchar;
type gid_t = libc::gid_t;
type uid_t = libc::uid_t;

extern "C" {
	fn proc_pidfdinfo(
		pid: int, fd: int, flavor: int, buffer: *mut libc::c_void, buffersize: int,
	) -> int;
}

const PROC_PIDFDSOCKETINFO: int = 3;
const TSI_T_NTIMERS: usize = 4;
const SOCKINFO_TCP: int = 2;

#[derive(Copy, Clone)]
#[repr(C)]
struct socket_fdinfo {
	pfi: proc_fileinfo,
	psi: socket_info,
}
#[derive(Copy, Clone)]
#[repr(C)]
struct proc_fileinfo {
	fi_openflags: u32,
	fi_status: u32,
	fi_offset: off_t,
	fi_type: i32,
	fi_guardflags: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
struct socket_info {
	soi_stat: vinfo_stat,
	soi_so: u64,  /* opaque handle of socket */
	soi_pcb: u64, /* opaque handle of protocol control block */
	soi_type: int,
	soi_protocol: int,
	soi_family: int,
	soi_options: short,
	soi_linger: short,
	soi_state: short,
	soi_qlen: short,
	soi_incqlen: short,
	soi_qlimit: short,
	soi_timeo: short,
	soi_error: u_short,
	soi_oobmark: u32,
	soi_rcv: sockbuf_info,
	soi_snd: sockbuf_info,
	soi_kind: int,
	rfu_1: u32, /* reserved */
	soi_proto: pri,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct vinfo_stat {
	vst_dev: u32,           /* [XSI] ID of device containing file */
	vst_mode: u16,          /* [XSI] Mode of file (see below) */
	vst_nlink: u16,         /* [XSI] Number of hard links */
	vst_ino: u64,           /* [XSI] File serial number */
	vst_uid: uid_t,         /* [XSI] User ID of the file */
	vst_gid: gid_t,         /* [XSI] Group ID of the file */
	vst_atime: i64,         /* [XSI] Time of last access */
	vst_atimensec: i64,     /* nsec of last access */
	vst_mtime: i64,         /* [XSI] Last data modification time */
	vst_mtimensec: i64,     /* last data modification nsec */
	vst_ctime: i64,         /* [XSI] Time of last status change */
	vst_ctimensec: i64,     /* nsec of last status change */
	vst_birthtime: i64,     /* File creation time(birth) */
	vst_birthtimensec: i64, /* nsec of File creation time */
	vst_size: off_t,        /* [XSI] file size, in bytes */
	vst_blocks: i64,        /* [XSI] blocks allocated for file */
	vst_blksize: i32,       /* [XSI] optimal blocksize for I/O */
	vst_flags: u32,         /* user defined flags for file */
	vst_gen: u32,           /* file generation number */
	vst_rdev: u32,          /* [XSI] Device ID */
	vst_qspare: [i64; 2],   /* RESERVED: DO NOT USE! */
}

#[derive(Copy, Clone)]
#[repr(C)]
struct sockbuf_info {
	sbi_cc: u32,
	sbi_hiwat: u32, /* SO_RCVBUF, SO_SNDBUF */
	sbi_mbcnt: u32,
	sbi_mbmax: u32,
	sbi_lowat: u32,
	sbi_flags: short,
	sbi_timeo: short,
}

#[derive(Copy, Clone)]
#[repr(C)]
union pri {
	pri_in: in_sockinfo,   /* SOCKINFO_IN */
	pri_tcp: tcp_sockinfo, /* SOCKINFO_TCP */
	// pri_un: un_sockinfo, /* SOCKINFO_UN */
	// pri_ndrv: ndrv_info, /* SOCKINFO_NDRV */
	// pri_kern_event: kern_event_info, /* SOCKINFO_KERN_EVENT */
	// pri_kern_ctl: kern_ctl_info, /* SOCKINFO_KERN_CTL */
	hack_to_avoid_copying_more_structs: [u8; 524],
}

#[derive(Copy, Clone)]
#[repr(C)]
struct in_sockinfo {
	insi_fport: int,  /* foreign port */
	insi_lport: int,  /* local port */
	insi_gencnt: u64, /* generation count of this instance */
	insi_flags: u32,  /* generic IP/datagram flags */
	insi_flow: u32,

	insi_vflag: u8,  /* ini_IPV4 or ini_IPV6 */
	insi_ip_ttl: u8, /* time to live proto */
	rfu_1: u32,      /* reserved */
	/* protocol dependent part */
	insi_faddr: addr, /* foreign host table entry */
	insi_laddr: addr, /* local host table entry */
	insi_v4: insi_v4,
	insi_v6: insi_v6,
}
impl fmt::Debug for in_sockinfo {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt.debug_struct("in_sockinfo")
			.field(
				"insi_fport",
				&u16::from_be(u16::try_from(self.insi_fport).unwrap()),
			)
			.field(
				"insi_lport",
				&u16::from_be(u16::try_from(self.insi_lport).unwrap()),
			)
			.field("insi_gencnt", &self.insi_gencnt)
			.field("insi_flags", &self.insi_flags)
			.field("insi_flow", &self.insi_flow)
			.field(
				"insi_vflag",
				&match self.insi_vflag {
					1 => "IPV4",
					2 => "IPV6",
					_ => "<unknown>", // unreachable!()
				},
			)
			.field("insi_ip_ttl", &self.insi_ip_ttl)
			.field("rfu_1", &self.rfu_1)
			.finish()
	}
}

#[derive(Copy, Clone)]
#[repr(C)]
struct insi_v4 {
	in4_tos: u_char, /* type of service */
}

#[derive(Copy, Clone)]
#[repr(C)]
struct insi_v6 {
	in6_hlim: u8,
	in6_cksum: int,
	in6_ifindex: u_short,
	in6_hops: short,
}

#[derive(Copy, Clone)]
#[repr(C)]
union addr {
	ina_46: in4in6_addr,
	ina_6: libc::in6_addr,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct in4in6_addr {
	i46a_pad32: [u32; 3],
	i46a_addr4: libc::in_addr,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tcp_sockinfo {
	tcpsi_ini: in_sockinfo,
	tcpsi_state: int,
	tcpsi_timer: [int; TSI_T_NTIMERS],
	tcpsi_mss: int,
	tcpsi_flags: u32,
	rfu_1: u32,    /* reserved */
	tcpsi_tp: u64, /* opaque handle of TCP protocol control block */
}
impl fmt::Debug for tcp_sockinfo {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		let Self {
			tcpsi_ini,
			tcpsi_state,
			tcpsi_timer,
			tcpsi_mss,
			tcpsi_flags,
			rfu_1,
			tcpsi_tp,
		} = self;
		fmt.debug_struct("tcp_sockinfo")
			.field("tcpsi_ini", tcpsi_ini)
			.field(
				"tcpsi_state",
				&(*tcpsi_state)
					.try_into()
					.ok()
					.and_then(state_to_name)
					.unwrap_or("<unknown>"),
			)
			.field("tcpsi_timer", tcpsi_timer)
			.field("tcpsi_mss", tcpsi_mss)
			.field("tcpsi_flags", tcpsi_flags)
			.field("rfu_1", rfu_1)
			.field("tcpsi_tp", tcpsi_tp)
			.finish()
	}
}
fn state_to_name(state: u8) -> Option<&'static str> {
	Some(match state {
		0 => "CLOSED",       /* closed */
		1 => "LISTEN",       /* listening for connection */
		2 => "SYN_SENT",     /* active, have sent syn */
		3 => "SYN_RECEIVED", /* have send and received syn */
		4 => "ESTABLISHED",  /* established */
		5 => "_CLOSE_WAIT",  /* rcvd fin, waiting for close */
		6 => "FIN_WAIT_1",   /* have closed, sent fin */
		7 => "CLOSING",      /* closed xchd FIN; await FIN ACK */
		8 => "LAST_ACK",     /* had fin and close; await FIN ACK */
		9 => "FIN_WAIT_2",   /* have closed, fin is acked */
		10 => "TIME_WAIT",   /* in 2*msl quiet wait after close */
		11 => "RESERVED",    /* pseudo state: reserved */
		_ => return None,
	})
}
