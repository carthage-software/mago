//! Terminal worker-state reduction carried over ordinary extension requests.

use crate::PayloadReader;
use crate::PayloadWriter;
use crate::ProtocolError;
use crate::WorkerError;

const MAGIC: [u8; 4] = *b"MEXT";
const MAJOR: u16 = 1;
const MINOR: u16 = 0;
const HEADER_LENGTH: usize = 12;
const COLLECT_REQUEST: u16 = 1;
const REDUCE_REQUEST: u16 = 2;
const COLLECT_RESPONSE: u16 = 0x8001;
const REDUCE_RESPONSE: u16 = 0x8002;
const MAXIMUM_REDUCERS: usize = 0x0000_4000;

pub(crate) fn collect_request() -> Vec<u8> {
    message_writer(COLLECT_REQUEST, HEADER_LENGTH).finish()
}

pub(crate) fn decode_collect_response(worker: usize, payload: &[u8]) -> Result<bool, WorkerError> {
    let mut reader = message_reader(worker, payload, COLLECT_RESPONSE)?;
    let count =
        reader.read_count("worker reducers", MAXIMUM_REDUCERS).map_err(|error| invalid_response(worker, error))?;
    let mut previous = None;
    for _ in 0..count {
        let index =
            reader.read_u32("worker reducer extension index").map_err(|error| invalid_response(worker, error))?;
        if previous.is_some_and(|previous| previous >= index) {
            return Err(invalid_response(worker, "worker reducer extension indices are not strictly increasing"));
        }

        previous = Some(index);
        let _payload = reader.read_bytes("worker reducer payload").map_err(|error| invalid_response(worker, error))?;
    }

    reader.finish().map_err(|error| invalid_response(worker, error))?;
    Ok(count != 0)
}

pub(crate) fn reduce_request(
    worker: usize,
    responses: &[Vec<u8>],
    maximum_payload_size: usize,
) -> Result<Vec<u8>, WorkerError> {
    let capacity = HEADER_LENGTH
        .checked_add(4)
        .and_then(|capacity| {
            responses.iter().try_fold(capacity, |capacity, response| capacity.checked_add(4 + response.len()))
        })
        .unwrap_or(usize::MAX);
    if capacity > maximum_payload_size || capacity > u32::MAX as usize {
        return Err(WorkerError::Protocol {
            worker,
            source: ProtocolError::FrameTooLarge { length: capacity, maximum: maximum_payload_size },
        });
    }

    let mut writer = message_writer(REDUCE_REQUEST, capacity);
    writer.write_length(responses.len()).map_err(|error| invalid_response(worker, error))?;
    for response in responses {
        writer.write_bytes(response).map_err(|error| invalid_response(worker, error))?;
    }

    Ok(writer.finish())
}

pub(crate) fn decode_reduce_response(worker: usize, payload: &[u8]) -> Result<(), WorkerError> {
    message_reader(worker, payload, REDUCE_RESPONSE)?.finish().map_err(|error| invalid_response(worker, error))
}

fn message_writer(kind: u16, capacity: usize) -> PayloadWriter {
    let mut writer = PayloadWriter::with_capacity(capacity);
    writer.write_raw(&MAGIC);
    writer.write_u16(MAJOR);
    writer.write_u16(MINOR);
    writer.write_u16(kind);
    writer.write_u16(0);
    writer
}

fn message_reader(worker: usize, payload: &[u8], expected_kind: u16) -> Result<PayloadReader<'_>, WorkerError> {
    let mut reader = PayloadReader::new(payload);
    let magic = reader.read_array::<4>("worker management magic").map_err(|error| invalid_response(worker, error))?;
    if magic != MAGIC {
        return Err(invalid_response(worker, "invalid worker management message magic"));
    }

    let major = reader.read_u16("worker management protocol major").map_err(|error| invalid_response(worker, error))?;
    let minor = reader.read_u16("worker management protocol minor").map_err(|error| invalid_response(worker, error))?;
    if major != MAJOR {
        return Err(invalid_response(
            worker,
            format!("unsupported worker management protocol version {major}.{minor}"),
        ));
    }

    let kind = reader.read_u16("worker management message kind").map_err(|error| invalid_response(worker, error))?;
    if kind != expected_kind {
        return Err(invalid_response(
            worker,
            format!("expected worker management message kind {expected_kind}, received {kind}"),
        ));
    }

    let reserved =
        reader.read_u16("worker management reserved bits").map_err(|error| invalid_response(worker, error))?;
    if reserved != 0 {
        return Err(invalid_response(worker, format!("worker management reserved bits are non-zero: {reserved:#06x}")));
    }

    Ok(reader)
}

fn invalid_response(worker: usize, error: impl std::fmt::Display) -> WorkerError {
    WorkerError::Reduction { worker, message: error.to_string() }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn validates_collection_responses_and_encodes_the_ordered_batch() {
        let mut first = message_writer(COLLECT_RESPONSE, 32);
        first.write_u32(1);
        first.write_u32(2);
        first.write_bytes(b"first").unwrap();
        let first = first.finish();
        let mut second = message_writer(COLLECT_RESPONSE, 32);
        second.write_u32(1);
        second.write_u32(2);
        second.write_bytes(b"second").unwrap();
        let second = second.finish();

        assert!(decode_collect_response(0, &first).unwrap());
        assert!(decode_collect_response(1, &second).unwrap());

        let request = reduce_request(0, &[first.clone(), second.clone()], usize::MAX).unwrap();
        let mut reader = message_reader(0, &request, REDUCE_REQUEST).unwrap();
        assert_eq!(reader.read_u32("responses").unwrap(), 2);
        assert_eq!(reader.read_bytes("first").unwrap(), first);
        assert_eq!(reader.read_bytes("second").unwrap(), second);
        reader.finish().unwrap();
    }

    #[test]
    fn rejects_oversized_reduction_batches() {
        let error = reduce_request(4, &[vec![0; 32]], 16).unwrap_err();
        assert!(matches!(error, WorkerError::Protocol { worker: 4, .. }));
    }
}
