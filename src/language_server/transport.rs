//! Bound incoming frames and report transport failures to the session.

use std::io;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use tokio::io::AsyncRead;
use tokio::io::ReadBuf;

pub(super) use self::observed::Observed;

mod observed;

const HEADER_LIMIT: usize = 8 * 1024;
const BODY_LIMIT: usize = 16 * 1024 * 1024;

pub(super) struct FrameReader<R> {
    inner: R,
    input: [u8; HEADER_LIMIT],
    start: usize,
    end: usize,
    header: Vec<u8>,
    sent: usize,
    remaining: Option<usize>,
}

impl<R> FrameReader<R> {
    pub(super) fn new(inner: R) -> Self {
        Self { inner, input: [0; HEADER_LIMIT], start: 0, end: 0, header: Vec::new(), sent: 0, remaining: None }
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn content_length(header: &[u8]) -> io::Result<usize> {
    if !header.is_ascii() {
        return Err(invalid("language server headers must be ASCII"));
    }
    let header = std::str::from_utf8(header).map_err(|_| invalid("invalid language server header"))?;
    let mut length = None;
    let mut content_type = false;
    for line in header[..header.len() - 4].split("\r\n") {
        let (name, value) = line.split_once(':').ok_or_else(|| invalid("invalid language server header"))?;
        if name.is_empty()
            || !name.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte))
            || value.bytes().any(|byte| byte.is_ascii_control() && byte != b'\t')
        {
            return Err(invalid("invalid language server header"));
        }
        let value = value.trim();
        if name.eq_ignore_ascii_case("Content-Length") {
            if length.is_some() || value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(invalid("invalid or duplicate Content-Length"));
            }
            let parsed = value.parse::<usize>().map_err(|_| invalid("invalid Content-Length"))?;
            if parsed > BODY_LIMIT {
                return Err(invalid("language server frame exceeds 16 MiB"));
            }
            length = Some(parsed);
        } else if name.eq_ignore_ascii_case("Content-Type") {
            if content_type {
                return Err(invalid("duplicate Content-Type"));
            }
            content_type = true;
            for parameter in value.split(';').skip(1) {
                if let Some((name, charset)) = parameter.trim().split_once('=')
                    && name.trim().eq_ignore_ascii_case("charset")
                    && !["utf-8", "utf8"].iter().any(|encoding| charset.trim().eq_ignore_ascii_case(encoding))
                {
                    return Err(invalid("language server content must use UTF-8"));
                }
            }
        }
    }
    length.ok_or_else(|| invalid("missing Content-Length"))
}

impl<R: AsyncRead + Unpin> AsyncRead for FrameReader<R> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, output: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if output.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }
        loop {
            if let Some(remaining) = this.remaining {
                if this.sent < this.header.len() {
                    let count = output.remaining().min(this.header.len() - this.sent);
                    output.put_slice(&this.header[this.sent..this.sent + count]);
                    this.sent += count;
                    return Poll::Ready(Ok(()));
                }
                this.header.clear();
                this.sent = 0;
                if remaining == 0 {
                    this.remaining = None;
                    continue;
                }
                if this.start < this.end {
                    let count = output.remaining().min(remaining).min(this.end - this.start);
                    output.put_slice(&this.input[this.start..this.start + count]);
                    this.start += count;
                    this.remaining = Some(remaining - count);
                    return Poll::Ready(Ok(()));
                }
            } else {
                while this.start < this.end {
                    this.header.push(this.input[this.start]);
                    this.start += 1;
                    if this.header.ends_with(b"\r\n\r\n") {
                        let length = content_length(&this.header)?;
                        // Tower accepts only two case-sensitive headers. Pass it one checked length.
                        this.header = format!("Content-Length: {length}\r\n\r\n").into_bytes();
                        this.remaining = Some(length);
                        break;
                    }
                    if this.header.len() >= HEADER_LIMIT {
                        return Poll::Ready(Err(invalid("language server headers exceed 8 KiB")));
                    }
                }
                if this.remaining.is_some() {
                    continue;
                }
            }

            let mut input = ReadBuf::new(&mut this.input);
            match Pin::new(&mut this.inner).poll_read(cx, &mut input) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
                Poll::Ready(Ok(())) => {}
            }
            this.start = 0;
            this.end = input.filled().len();
            if this.end == 0 {
                return if this.remaining.is_some() || !this.header.is_empty() {
                    Poll::Ready(Err(io::Error::new(io::ErrorKind::UnexpectedEof, "incomplete language server frame")))
                } else {
                    Poll::Ready(Ok(()))
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;

    use super::*;

    fn frame(body: &str) -> String {
        format!("Content-Length: {}\r\n\r\n{body}", body.len())
    }

    #[tokio::test]
    async fn split_headers_bodies_and_coalesced_frames_keep_their_bytes() {
        let input = format!("{}{}{}", frame("héllo"), frame(""), frame("next\r\n\r\nbody"));
        for chunk in [1, 2, 7, input.len()] {
            let (mut writer, reader) = tokio::io::duplex(32);
            let bytes = input.clone();
            let send = tokio::spawn(async move {
                for part in bytes.as_bytes().chunks(chunk) {
                    writer.write_all(part).await.unwrap();
                    tokio::task::yield_now().await;
                }
            });
            let mut actual = Vec::new();
            FrameReader::new(reader).read_to_end(&mut actual).await.unwrap();
            assert_eq!(actual, input.as_bytes());
            send.await.unwrap();
        }
    }

    #[tokio::test]
    async fn accepts_header_case_extensions_and_utf8_aliases() {
        for encoding in ["utf-8", "utf8", "UTF-8"] {
            let input = format!(
                "content-length: 2\r\nX-Trace: ignored\r\nContent-Type: application/vscode-jsonrpc; charset={encoding}\r\n\r\n{{}}"
            );
            let mut actual = String::new();
            FrameReader::new(input.as_bytes()).read_to_string(&mut actual).await.unwrap();
            assert_eq!(actual, frame("{}"));
        }
    }

    #[tokio::test]
    async fn rejects_bad_headers_before_forwarding_them() {
        for input in [
            "X-Trace: missing length\r\n\r\n".to_owned(),
            "Content-Length: +1\r\n\r\n".to_owned(),
            "Content-Length: -1\r\n\r\n".to_owned(),
            "Content-Length: 1\r\nContent-Length: 2\r\n\r\n".to_owned(),
            "Content-Length: 184467440737095516160\r\n\r\n".to_owned(),
            "Content-Length: 16777217\r\n\r\n".to_owned(),
            "Content-Length: 1\r\nContent-Type: application/vscode-jsonrpc; charset=latin1\r\n\r\n".to_owned(),
            "Content-Length: 1\r\nX-Trace: é\r\n\r\n".to_owned(),
            "Content-Length: 1\r\nBad Header: value\r\n\r\n".to_owned(),
            "x".repeat(HEADER_LIMIT),
        ] {
            let mut actual = Vec::new();
            let error = FrameReader::new(input.as_bytes()).read_to_end(&mut actual).await.unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{input}");
            assert!(actual.is_empty());
        }
        assert_eq!(content_length(b"Content-Length: 16777216\r\n\r\n").unwrap(), BODY_LIMIT);
    }

    #[tokio::test]
    async fn rejects_truncated_headers_and_bodies() {
        for input in ["Content-Length: 2\r\n", "Content-Length: 2\r\n\r\nx"] {
            let error = FrameReader::new(input.as_bytes()).read_to_end(&mut Vec::new()).await.unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
        }
    }

    #[tokio::test]
    async fn frame_failure_stops_serve_even_if_output_waits() {
        let (reader, _keep_open) = tokio::io::duplex(1);
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            crate::language_server::serve(
                b"Content-Length: 16777217\r\n\r\n".as_slice(),
                reader,
                crate::language_server::ServerConfig::default(),
            ),
        )
        .await
        .unwrap();
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
    }

    #[tokio::test]
    async fn output_failure_stops_serve_even_if_input_waits() {
        let (mut client, reader) = tokio::io::duplex(1024);
        let (writer, closed) = tokio::io::duplex(1);
        drop(closed);
        client.write_all(frame(r#"{"jsonrpc":"2.0","id":1,"method":"unknown"}"#).as_bytes()).await.unwrap();
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            crate::language_server::serve(reader, writer, crate::language_server::ServerConfig::default()),
        )
        .await
        .unwrap();
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::BrokenPipe);
    }

    #[tokio::test]
    async fn early_exit_and_eof_fail_the_session() {
        for input in [String::new(), frame(r#"{"jsonrpc":"2.0","method":"exit"}"#)] {
            let result = tokio::time::timeout(
                Duration::from_secs(5),
                crate::language_server::serve(
                    input.as_bytes(),
                    tokio::io::sink(),
                    crate::language_server::ServerConfig::default(),
                ),
            )
            .await
            .unwrap();
            assert!(result.is_err());
        }
    }
}
