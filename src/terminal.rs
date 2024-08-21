use std::os::unix::io::AsRawFd;
use Color::*;

#[cfg(target_os = "macos")]
pub fn empty_termios() -> libc::termios {
    libc::termios {
        c_ispeed: 0, c_ospeed: 0, c_iflag: 0, c_oflag: 0, c_cflag: 0, c_lflag: 0, 
        c_cc: [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
    }
}

#[cfg(target_os = "linux")]
pub fn empty_termios() -> libc::termios {
    libc::termios {
        c_ispeed: 0, c_ospeed: 0, c_iflag: 0, c_oflag: 0, c_cflag: 0, c_lflag: 0, 
        c_line: 0,
        c_cc: [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, ],
    }
}

// copied and refactored from: https://docs.rs/console/0.15.0/src/console/unix_term.rs.html#87
pub fn get_input() -> Option<[u8; 4]> {
    let tty_f;
    let fd = unsafe {
        if libc::isatty(libc::STDIN_FILENO) == 1 {
            libc::STDIN_FILENO
        } else {
            tty_f = std::fs::File::open("/dev/tty").unwrap(); // todo: unsafe
            tty_f.as_raw_fd()
        }
    };

    // todo: can't get to type umlaut on mac

    let mut pollfd = libc::pollfd { fd, events: libc::POLLIN, revents: 0, };
    let ret = unsafe { libc::poll(&mut pollfd as *mut _, 1, 0) };
    if ret < 0 { return None }

    if pollfd.revents & libc::POLLIN != 0 {
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        let read = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut _, 4) };
        if read < 0 {
            None
        } else if read == 0 {
            None
        } else if buf[0] == b'\x03' {
            None
        } else {
            Some(buf)
            // Some(String::from_utf8(buf.to_vec()).unwrap().chars().next().unwrap())
        }
    } else {
        None
    }
}

pub fn as_char(bytes: [u8; 4]) -> char {
    String::from_utf8(bytes.to_vec()).unwrap().chars().next().unwrap()
}

pub fn clear_window() {
    // todo: this does not delete the history on macos
    print!("{esc}c", esc = 27 as char);
}

pub fn prepare_terminal() {
    unsafe {
        let mut tty = empty_termios();
        libc::tcgetattr(0, &mut tty);
        tty.c_lflag &= !libc::ICANON;
        tty.c_lflag &= !libc::ECHO;
        // tty.c_lflag &= !libc::ISIG;
        libc::tcsetattr(0, libc::TCSANOW, &tty);
    }
}

pub fn restore_terminal() {
    unsafe {
        let mut tty = empty_termios();
        libc::tcgetattr(0, &mut tty);
        tty.c_lflag |= libc::ICANON;
        tty.c_lflag |= libc::ECHO;
        // tty.c_lflag |= libc::ISIG;
        libc::tcsetattr(0, libc::TCSANOW, &tty);
    }
}

pub fn get_window_size() -> (usize, usize) {
    unsafe {
        let mut size = libc::winsize { ws_col: 0, ws_row: 0, ws_xpixel: 0, ws_ypixel: 0 };
        libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size);
        return (size.ws_col as usize, size.ws_row as usize);
    }
}

pub fn color_start(color: &Color) -> String {
    let addition = if *color < BlackBg {
        30
    } else if *color < BlackBright {
        40 - (BlackBg as i32)
    } else if *color < BlackBrightBg {
        90 - (BlackBright as i32)
    } else {
        100 - (BlackBrightBg as i32)
    };
    let code = addition + color.clone() as i32;
    return format!("\x1b[{}m", code);
}

pub const COLOR_END: &str = "\x1b[0m";

#[allow(dead_code)]
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum Color {
    Black, Red, Green, Yellow, Blue, Magenta, Cyan, White,
    BlackBg, RedBg, GreenBg, YellowBg, BlueBg, MagentaBg, CyanBg, WhiteBg,
    BlackBright, RedBright, GreenBright, YellowBright, BlueBright, MagentaBright, CyanBright, WhiteBright,
    BlackBrightBg, RedBrightBg, GreenBrightBg, YellowBrightBg, BlueBrightBg, MagentaBrightBg, CyanBrightBg, WhiteBrightBg,
}
