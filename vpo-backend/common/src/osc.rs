use std::{borrow::Cow, ffi::CStr, io::Write};

use memchr;

#[derive(Default)]
pub enum OscView<'a> {
    Message(OscMessageView<'a>),
    Bundle(OscBundleView<'a>),
    #[default]
    None,
}

impl<'a> OscView<'a> {
    pub fn all_messages<F>(&'a self, mut f: F)
    where
        F: FnMut(usize, Option<OscTime>, &OscMessageView<'a>),
    {
        match self {
            OscView::Bundle(bundle) => {
                bundle.all_messages(|offset_from_start, time, message| f(offset_from_start, Some(time), &message))
            }
            OscView::Message(message) => {
                f(0, None, message);
            }
            _ => {}
        }
    }
}

impl core::fmt::Debug for OscView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OscView {{ messages: [")?;

        let mut any_messages = false;

        self.all_messages(|_, _, message| {
            let _ = write!(f, "\n{:?}", message);

            any_messages = true;
        });

        if any_messages {
            writeln!(f)?;
        }

        write!(f, "] }}")
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct OscTime {
    pub seconds: u32,
    pub fractional: u32,
}

#[derive(Debug, PartialEq)]
pub enum OscArg<'a> {
    Integer(i32),
    Float(f32),
    String(Cow<'a, CStr>),
    Blob(Cow<'a, [u8]>),
    True,
    False,
    Null,
    Impulse,
    Timetag(OscTime),
}

impl<'a> OscArg<'a> {
    pub fn type_as_byte(&self) -> u8 {
        match self {
            OscArg::Integer(_) => b'i',
            OscArg::Float(_) => b'f',
            OscArg::String(_) => b's',
            OscArg::Blob(_) => b'b',
            OscArg::True => b'T',
            OscArg::False => b'F',
            OscArg::Null => b'N',
            OscArg::Impulse => b'I',
            OscArg::Timetag(_) => b't',
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<usize, std::io::Error> {
        match self {
            OscArg::Integer(int) => {
                writer.write_all(&int.to_be_bytes())?;
                Ok(4)
            }
            OscArg::Float(float) => {
                writer.write_all(&float.to_be_bytes())?;
                Ok(4)
            }
            OscArg::String(str) => {
                writer.write_all(str.to_bytes_with_nul())?;

                let mut written = str.to_bytes_with_nul().len();
                written += write_padding_32(written, writer)?;

                Ok(written)
            }
            OscArg::Blob(blob) => {
                let mut written = 0;

                writer.write_all(&(blob.len() as u32).to_be_bytes())?;
                written += 4;

                writer.write_all(blob)?;
                written += blob.len();

                written += write_padding_32(written, writer)?;

                Ok(written)
            }
            OscArg::True | OscArg::False | OscArg::Null | OscArg::Impulse => Ok(0),
            OscArg::Timetag(time) => {
                writer.write_all(&time.seconds.to_be_bytes())?;
                writer.write_all(&time.fractional.to_be_bytes())?;

                Ok(8)
            }
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            OscArg::Integer(x) => Some(*x),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            OscArg::Float(x) => Some(*x),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&CStr> {
        match self {
            OscArg::String(x) => Some(x.as_ref()),
            _ => None,
        }
    }

    pub fn as_blob(&self) -> Option<&[u8]> {
        match self {
            OscArg::Blob(x) => Some(x.as_ref()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            OscArg::True => Some(true),
            OscArg::False => Some(false),
            _ => None,
        }
    }

    pub fn as_null(&self) -> Option<()> {
        match &self {
            OscArg::Null => Some(()),
            _ => None,
        }
    }

    pub fn as_impulse(&self) -> Option<()> {
        match self {
            OscArg::Impulse => Some(()),
            _ => None,
        }
    }

    pub fn as_timetag(&self) -> Option<OscTime> {
        match self {
            OscArg::Timetag(t) => Some(t.clone()),
            _ => None,
        }
    }
}

fn padding_32(pos: usize) -> usize {
    3 - (pos % 4)
}

fn write_str_padded<W: Write>(str: &CStr, writer: &mut W) -> Result<usize, std::io::Error> {
    let bytes = str.to_bytes_with_nul();

    writer.write_all(bytes)?;

    let mut written = bytes.len();
    written += write_padding_32(written, writer)?;

    Ok(written)
}

fn write_padding_32<W: Write>(written_so_far: usize, writer: &mut W) -> Result<usize, std::io::Error> {
    let mut written = 0;
    let needed_padding = padding_32(written_so_far + 3);

    for _ in 0..needed_padding {
        writer.write_all(&[0])?;
        written += 1;
    }

    Ok(written)
}

/// Returns the string as `&[u8]`, including a single null terminator
fn read_string<'a>(message: &'a [u8], cursor: &mut usize) -> Option<&'a CStr> {
    let pos = *cursor;

    let message_null = memchr::memchr(0, &message[pos..])? + pos;
    let end_of_message = message_null + padding_32(message_null);

    *cursor = end_of_message + 1;

    let cstr = CStr::from_bytes_with_nul(&message[pos..(message_null + 1)]).ok()?;

    Some(cstr)
}

fn read_u32(message: &[u8], cursor: &mut usize) -> u32 {
    let pos = *cursor;

    *cursor += 4;

    u32::from_be_bytes([message[pos + 0], message[pos + 1], message[pos + 2], message[pos + 3]])
}

fn read_i32(message: &[u8], cursor: &mut usize) -> i32 {
    let pos = *cursor;

    *cursor += 4;

    i32::from_be_bytes([message[pos + 0], message[pos + 1], message[pos + 2], message[pos + 3]])
}

fn read_f32(message: &[u8], cursor: &mut usize) -> f32 {
    let pos = *cursor;

    *cursor += 4;

    f32::from_be_bytes([message[pos + 0], message[pos + 1], message[pos + 2], message[pos + 3]])
}

fn read_timetag(message: &[u8], cursor: &mut usize) -> OscTime {
    let seconds = read_u32(&message, cursor);
    let fractional = read_u32(&message, cursor);

    OscTime { seconds, fractional }
}

impl<'a> OscView<'a> {
    /// Create a view of an OSC message.
    ///
    /// Returns a bundle or message.
    pub fn new(message: &'a [u8]) -> Option<OscView> {
        // all osc messages must be aligned
        if message.len() % 4 != 0 || message.len() == 0 {
            return None;
        }

        if message.len() > 8 && &message[0..8] == b"#bundle\0" {
            Some(OscView::Bundle(OscBundleView {
                content: (&message[8..]).into(),
            }))
        } else if message[0] == b'/' {
            Some(OscView::Message(OscMessageView::new(message)?))
        } else {
            None
        }
    }
}

/// An OSC bundle. Iterate over the messages using `all_messages`.
pub struct OscBundleView<'a> {
    content: Cow<'a, [u8]>,
}

fn handle_element<'a, F>(bytes: &'a [u8], offset_from_start: usize, f: &mut F)
where
    F: FnMut(usize, OscTime, OscMessageView<'a>),
{
    let mut cursor = 0;

    let timetag = read_timetag(bytes, &mut cursor);

    while cursor < bytes.len() {
        let elem_len = read_u32(bytes, &mut cursor) as usize;
        let elem = &bytes[cursor..(cursor + elem_len)];

        if let Some(message) = OscMessageView::new(elem) {
            (f)(offset_from_start + cursor, timetag.clone(), message);
        } else if elem.len() >= 8 && &elem[0..8] == b"#bundle\0" {
            handle_element(&elem[8..], offset_from_start + cursor + 8, f);
        }

        cursor += elem_len;
    }
}

impl<'a> OscBundleView<'a> {
    pub fn all_messages<F>(&'a self, mut f: F)
    where
        F: FnMut(usize, OscTime, OscMessageView<'a>),
    {
        let bytes = self.content.as_ref();

        handle_element(bytes, 8, &mut f);
    }
}

pub struct OscMessageView<'a> {
    address: &'a CStr,
    type_tag: &'a CStr,
    arguments: &'a [u8],
    bytes: &'a [u8],
}

impl core::fmt::Debug for OscMessageView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[address: ")?;

        match self.address.to_str() {
            Ok(s) => write!(f, "\"{}\"", s)?,
            Err(_) => write!(f, "invalid UTF-8")?,
        }

        write!(f, ", arguments: [")?;

        let mut first_time = true;
        for arg in self.arg_iter() {
            if !first_time {
                write!(f, ", ")?;
            }

            write!(f, "{:?}", arg)?;

            first_time = false;
        }

        write!(f, "]]")
    }
}

impl<'a> OscMessageView<'a> {
    fn new(message: &'a [u8]) -> Option<OscMessageView<'a>> {
        let mut cursor = 0;

        let address = read_string(message, &mut cursor)?;
        let type_tag = read_string(message, &mut cursor)?;

        if type_tag.to_bytes()[0] != b',' {
            return None;
        }

        Some(OscMessageView {
            address: address,
            type_tag: type_tag,
            arguments: &message[cursor..],
            bytes: message,
        })
    }

    pub fn arg_iter(&'a self) -> ArgsIter<'a> {
        ArgsIter {
            message: self,
            type_tag_cursor: 1,
            arg_cursor: 0,
        }
    }

    pub fn address(&self) -> &CStr {
        &self.address
    }

    pub fn type_tag(&self) -> &CStr {
        &self.type_tag
    }

    pub fn arguments(&self) -> &[u8] {
        &self.arguments
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes
    }
}

pub struct ArgsIter<'a> {
    message: &'a OscMessageView<'a>,
    type_tag_cursor: usize,
    arg_cursor: usize,
}

impl<'a> Iterator for ArgsIter<'a> {
    type Item = OscArg<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let type_tag = self.message.type_tag.to_bytes();
        let args = &self.message.arguments;

        if self.type_tag_cursor >= type_tag.len() {
            return None;
        }

        let res = match type_tag[self.type_tag_cursor] {
            b'i' => OscArg::Integer(read_i32(&args, &mut self.arg_cursor)),
            b'f' => OscArg::Float(read_f32(&args, &mut self.arg_cursor)),
            b's' => OscArg::String(read_string(&args, &mut self.arg_cursor)?.into()),
            b'b' => {
                let blob_bytes = read_u32(&args, &mut self.arg_cursor) as usize;

                let blob = &args[self.arg_cursor..(self.arg_cursor + blob_bytes)];
                let padding = padding_32(self.arg_cursor + blob_bytes - 1);

                self.arg_cursor += blob_bytes + padding;

                OscArg::Blob(blob.into())
            }
            b'T' => OscArg::True,
            b'F' => OscArg::False,
            b'N' => OscArg::Null,
            b'I' => OscArg::Impulse,
            b't' => OscArg::Timetag(read_timetag(&args, &mut self.arg_cursor)),
            _ => return None,
        };

        self.type_tag_cursor += 1;

        Some(res)
    }
}

impl<'a> ExactSizeIterator for ArgsIter<'a> {
    fn len(&self) -> usize {
        self.message.type_tag().to_bytes().len() - 1
    }
}
pub enum OscWriterMode {
    DryRun,
    ActualRun,
}

pub fn write_osc_message<W: Write>(writer: &mut W, address: &CStr, args: &[OscArg]) -> Result<usize, std::io::Error> {
    let mut cursor = 0;

    cursor += write_str_padded(address, writer)?;
    writer.write_all(&[b','])?;
    cursor += 1;

    for arg in args {
        writer.write_all(&[arg.type_as_byte()])?;
        cursor += 1;
    }

    writer.write_all(&[b'\0'])?;
    cursor += 1;

    cursor += write_padding_32(cursor, writer)?;

    for arg in args {
        cursor += arg.write(writer)?;
    }

    Ok(cursor)
}

pub fn write_osc_message_prepend_len<W: Write>(
    writer: &mut W,
    address: &CStr,
    args: &[OscArg],
) -> Result<usize, std::io::Error> {
    let msg_len = write_osc_message(&mut NoopWriter {}, address, args).unwrap();

    let mut cursor = 0;

    writer.write_all(&(msg_len as u32).to_be_bytes())?;
    cursor += 4;

    cursor += write_osc_message(writer, address, args)?;

    Ok(cursor)
}

pub struct NoopWriter {}

impl Write for NoopWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct BundleWriter<'writer, W: Write> {
    writer: Option<&'writer mut W>,
    written: usize,
}

impl<'writer, W: Write> BundleWriter<'writer, W> {
    pub fn start(mut writer: Option<&'writer mut W>, timetag: OscTime) -> std::io::Result<BundleWriter<'writer, W>> {
        if let Some(writer) = writer.as_mut() {
            writer.write_all(b"#bundle\0")?;
            OscArg::Timetag(timetag).write(writer)?;
        };

        Ok(BundleWriter {
            writer: writer,
            written: 16, // b"#bundle\0" + 8 byte timetag
        })
    }

    pub fn write_message(&mut self, address: &CStr, args: &[OscArg]) -> std::io::Result<()> {
        let mut noop_writer = NoopWriter {};

        let message_len = write_osc_message(&mut noop_writer, address, args)?;

        if let Some(writer) = self.writer.as_mut() {
            writer.write_all(&(message_len as u32).to_be_bytes())?;

            write_osc_message(writer, address, args)?;
        }

        self.written += 4;
        self.written += message_len;

        Ok(())
    }

    pub fn get_written(&self) -> usize {
        self.written
    }
}

#[test]
fn test_message_parsing() {
    // https://opensoundcontrol.stanford.edu/spec-1_0-examples.html

    let msg1 = vec![
        b'/', b'f', b'o', b'o', // /foo
        0x00, 0x00, 0x00, 0x00, // nulls
        b',', b'i', b'i', b's', // type tag (,iisff)
        b'f', b'f', 0x00, 0x00, // type tag + nulls
        0x00, 0x00, 0x03, 0xE8, // int32 1000
        0xFF, 0xFF, 0xFF, 0xFF, // int32 -1
        b'h', b'e', b'l', b'l', // string (hello)
        b'o', 0x00, 0x00, 0x00, // string + nulls
        0x3F, 0x9D, 0xF3, 0xB6, // float32 1.234
        0x40, 0xB5, 0xB2, 0x2D, // float32 5.678
    ];

    let view = OscView::new(&msg1[..]).unwrap();

    match view {
        OscView::Message(message) => {
            let mut iter = message.arg_iter();

            assert_eq!(iter.next(), Some(OscArg::Integer(1000)));
            assert_eq!(iter.next(), Some(OscArg::Integer(-1)));
            assert_eq!(iter.next(), Some(OscArg::String(cstr("hello\0").into())));
            assert_eq!(iter.next(), Some(OscArg::Float(1.234)));
            assert_eq!(iter.next(), Some(OscArg::Float(5.678)));
            assert_eq!(iter.next(), None);
        }
        _ => panic!("wrong type of message"),
    }

    // poor man's fuzzing
    for string in naughty_strings::BLNS {
        OscView::new(string.as_bytes());
    }
}

#[cfg(test)]
fn cstr(x: &str) -> &CStr {
    CStr::from_bytes_with_nul(x.as_bytes()).unwrap()
}

#[test]
fn test_message_generation() {
    let mut writer: Vec<u8> = vec![];

    write_osc_message(
        &mut writer,
        c"/foo/bar",
        &[
            OscArg::String(c"hello".into()),
            OscArg::Float(1.234),
            OscArg::Float(5.678),
        ],
    )
    .unwrap();

    let msg = writer;

    let view = OscView::new(&msg[..]).unwrap();

    match view {
        OscView::Message(message) => {
            let mut iter = message.arg_iter();

            assert_eq!(iter.next(), Some(OscArg::String(cstr("hello\0").into())));
            assert_eq!(iter.next(), Some(OscArg::Float(1.234)));
            assert_eq!(iter.next(), Some(OscArg::Float(5.678)));
            assert_eq!(iter.next(), None);
        }
        _ => panic!("wrong type of message"),
    }
}

#[test]
fn test_bundle_generation() {
    let mut writer = vec![];

    let mut bundle_writer = BundleWriter::start(
        Some(&mut writer),
        OscTime {
            seconds: 10,
            fractional: 23,
        },
    )
    .unwrap();

    bundle_writer
        .write_message(
            cstr("/foo/bar\0"),
            &[
                OscArg::String(cstr("hello\0").into()),
                OscArg::Float(1.234),
                OscArg::Float(5.678),
            ],
        )
        .unwrap();

    bundle_writer
        .write_message(
            cstr("/baz/quxley\0"),
            &[
                OscArg::Integer(123),
                OscArg::Float(1.234),
                OscArg::Float(5.678),
                OscArg::False,
                OscArg::True,
                OscArg::Impulse,
            ],
        )
        .unwrap();

    let msg = writer;

    let view = OscView::new(&msg[..]).unwrap();

    match view {
        OscView::Bundle(bundle) => {
            let mut i = 0;

            bundle.all_messages(|offset_from_start, time, message| {
                assert_eq!(
                    time,
                    OscTime {
                        seconds: 10,
                        fractional: 23,
                    },
                );

                let mut iter = message.arg_iter();

                match i {
                    0 => {
                        assert_eq!(offset_from_start, 8 + 8 + 4);

                        assert_eq!(message.address, cstr("/foo/bar\0"));

                        assert_eq!(iter.next(), Some(OscArg::String(cstr("hello\0").into())));
                        assert_eq!(iter.next(), Some(OscArg::Float(1.234)));
                        assert_eq!(iter.next(), Some(OscArg::Float(5.678)));
                        assert_eq!(iter.next(), None);
                    }
                    1 => {
                        assert_eq!(offset_from_start, 8 + 8 + 4 + 36 + 4);

                        assert_eq!(message.address, cstr("/baz/quxley\0"));

                        assert_eq!(iter.next(), Some(OscArg::Integer(123)));
                        assert_eq!(iter.next(), Some(OscArg::Float(1.234)));
                        assert_eq!(iter.next(), Some(OscArg::Float(5.678)));
                        assert_eq!(iter.next(), Some(OscArg::False));
                        assert_eq!(iter.next(), Some(OscArg::True));
                        assert_eq!(iter.next(), Some(OscArg::Impulse));
                        assert_eq!(iter.next(), None);
                    }
                    _ => panic!("only two messages"),
                }

                i += 1;
            });
        }
        _ => unreachable!("should be a bundle"),
    }
}
