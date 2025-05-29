use std::io;

use super::WirePacket;
use bytemuck::Pod;
use easy_ext::ext;
use pack1::U16LE;
use simple_coro::{Coro, CoroState, Handle};

type Buf<'a> = &'a mut Vec<u8>;

enum Req<'a> {
    GiveMe(usize),
    YourData(Buf<'a>),
}
enum Resp<'a> {
    ThereYouGo(Buf<'a>),
    Thanks,
}
type H<'a> = Handle<Req<'a>, Resp<'a>>;

macro_rules! gimme {
    ($pat:pat = $expr:expr) => {
        let $pat = $expr else {
            panic!("Contract breached")
        };
    };
}

#[ext]
impl<'a> H<'a> {
    async fn bytes_for<N: Pod>(&mut self) -> N {
        let n = size_of::<N>();
        gimme!(Resp::ThereYouGo(buf) = self.yield_value(Req::GiveMe(n)).await);
        let this = *bytemuck::from_bytes(buf);
        gimme!(Resp::Thanks = self.yield_value(Req::YourData(buf)).await);
        this
    }
}

impl<'a> WirePacket<'a> {
    async fn read_sansio(mut handle: H<'a>) -> Self {
        let flags: u8 = handle.bytes_for().await;
        let len: U16LE = handle.bytes_for().await;
        gimme!(Resp::ThereYouGo(data) = handle.yield_value(Req::GiveMe(len.get() as usize)).await);
        Self {
            flags,
            len: len.get(),
            data,
        }
    }

    fn write_sansio(&self, buf: &mut Vec<u8>) {
        buf.push(self.flags);
        buf.extend_from_slice(&self.len.to_le_bytes());
        buf.extend_from_slice(self.data);
    }
    /// Blockingly reads a packet from `read` using `buf` as auxiliary memory.
    ///
    /// The buffer will be modified and cleared.
    /// The contents read must be encoded using [`Self::write`].
    /// All language bindings implement this.
    pub fn read(buf: &'a mut Vec<u8>, read: &mut dyn io::Read) -> io::Result<Self> {
        let mut coro = Coro::from(Self::read_sansio);
        let mut buf = Some(buf);
        loop {
            coro = match coro.resume() {
                CoroState::Pending(c, n) => match n {
                    Req::GiveMe(n) => {
                        let buf = buf.take().unwrap();
                        buf.clear();
                        buf.resize(n, 0);
                        read.read_exact(buf)?;
                        c.send(Resp::ThereYouGo(buf))
                    }
                    Req::YourData(buf2) => {
                        buf = Some(buf2);
                        c.send(Resp::Thanks)
                    }
                },
                CoroState::Complete(this) => return Ok(this),
            }
        }
    }

    /// Blockingly write into `write` the contents of `Self`, using `buf` as auxiliary memory.
    ///
    /// The buffer will be modified and cleared.
    /// The contents written can then be decoded by [`Self::read`].
    /// All language bindings implement this.
    pub fn write(&self, buf: &mut Vec<u8>, write: &mut dyn io::Write) -> io::Result<()> {
        buf.clear();

        buf.reserve(128usize.saturating_sub(buf.capacity()));
        self.write_sansio(buf);
        write.write_all(buf.as_slice())?;
        Ok(())
    }
}
