<?php

declare(strict_types=1);

use Mago\Sdk\Exception\CancelledException;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Internal\Io\ResourceReader;
use Mago\Sdk\Internal\Io\ResourceWriter;
use Mago\Sdk\Internal\Protocol\Frame;
use Mago\Sdk\Internal\Protocol\FrameCodec;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\Internal\Protocol\PayloadWriter;
use Mago\Sdk\Internal\SignalCancellationToken;
use Mago\Sdk\Internal\Syntax\NodeStore;
use Mago\Sdk\Internal\Syntax\ResolvedNameStore;
use Mago\Sdk\Internal\Syntax\TriviaStore;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Span;
use Mago\Sdk\Syntax\NodeKind;
use Mago\Sdk\Syntax\SourceFile;
use Mago\Sdk\Syntax\TriviaKind;
use Revolt\EventLoop;

require_once __DIR__ . '/../../../vendor/autoload.php';

/**
 * @param non-empty-string $message
 * @mago-expect lint:no-boolean-flag-parameter
 */
function expect(bool $condition, string $message): void
{
    if (!$condition) {
        throw new RuntimeException($message);
    }
}

$writer = new PayloadWriter();
$writer->writeU8(7);
$writer->writeU16(0x0203);
$writer->writeU32(0x0405_0607);
$writer->writeU64(42);
$writer->writeBoolean(true);
$writer->writeBytes("bytes\0");
$writer->writeString('string');
$writer->writeOptionalString('optional');
$writer->writeOptionalString(null);

$reader = new PayloadReader($writer->finish());
expect($reader->readU8() === 7, 'u8 did not round-trip.');
expect($reader->readU16() === 0x0203, 'u16 did not round-trip.');
expect($reader->readU32() === 0x0405_0607, 'u32 did not round-trip.');
expect($reader->readU64() === 42, 'u64 did not round-trip.');
expect($reader->readBoolean(), 'boolean did not round-trip.');
expect($reader->readBytes() === "bytes\0", 'bytes did not round-trip.');
expect($reader->readString() === 'string', 'string did not round-trip.');
expect($reader->readOptionalString() === 'optional', 'optional string did not round-trip.');
expect($reader->readOptionalString() === null, 'absent string did not round-trip.');
$reader->finish();

$noNode = 4_294_967_295;
$nodeRecords =
    pack('CNNNNN', 0, 0, 10, $noNode, 1, $noNode)
    . pack('CNNNNN', 1, 1, 4, 0, $noNode, 2)
    . pack('CNNNNN', 1, 5, 8, 0, $noNode, $noNode);
$nodeStore = new NodeStore([NodeKind::Program, NodeKind::FunctionCall], $nodeRecords, 3);
$resolvedName = 'Psl\\Iter\\any';
$nameStarts = pack('N', 1);
$nameRecords = pack('NNNC', 4, 0, strlen($resolvedName), 0);
$nameStore = new ResolvedNameStore($nameStarts, $nameRecords, $resolvedName, 1);
$triviaStore = new TriviaStore(pack('CNN', 4, 0, 10), 1);
$sourceFile = new SourceFile(
    PHPVersion::fromParts(8, 3),
    'fixture.php',
    '0123456789',
    [1, 2],
    $nodeStore,
    $nameStore,
    $triviaStore,
);
$targets = $sourceFile->getTargetNodes();
expect(count($targets) === 2, 'Target nodes were not materialized.');
expect($targets[0]->kind === NodeKind::FunctionCall, 'The packed node kind was not decoded.');
expect($sourceFile->getChildren($sourceFile->getNode(0)) === $targets, 'Packed sibling links were not decoded.');
expect($sourceFile->getParent($targets[0])?->id === 0, 'The packed parent link was not decoded.');
expect($sourceFile->getText($targets[0]) === '123', 'Packed node spans did not select source text.');
expect($sourceFile->getResolvedName($targets[0])?->name === $resolvedName, 'The resolved name was not found.');
expect($sourceFile->getTrivia()[0]->kind === TriviaKind::DocBlockComment, 'Packed trivia was not decoded.');

$codec = new FrameCodec(1024);
$encoded = $codec->encode(Frame::response(42, "payload\0"));
$frameStreams = stream_socket_pair(STREAM_PF_UNIX, STREAM_SOCK_STREAM, STREAM_IPPROTO_IP);
if ($frameStreams === false) {
    throw new RuntimeException('Unable to create frame test streams.');
}

[$frameInput, $frameOutput] = $frameStreams;
fwrite($frameOutput, $encoded);
fclose($frameOutput);
$frameReader = new ResourceReader($frameInput);
$frame = $codec->read($frameReader);
$frameReader->close();
fclose($frameInput);
if ($frame === null) {
    throw new RuntimeException('Frame was not decoded.');
}

expect($frame->id === 42, 'Frame identifier did not round-trip.');
expect($frame->payload === "payload\0", 'Frame payload did not round-trip.');

$readerStreams = stream_socket_pair(STREAM_PF_UNIX, STREAM_SOCK_STREAM, STREAM_IPPROTO_IP);
if ($readerStreams === false) {
    throw new RuntimeException('Unable to create reader test streams.');
}

[$readerInput, $readerOutput] = $readerStreams;
$resourceReader = new ResourceReader($readerInput);
EventLoop::delay(0.001, static function () use ($readerOutput): void {
    fwrite($readerOutput, 'ready');
    fclose($readerOutput);
});
expect($resourceReader->readExactly(5) === 'ready', 'The resource reader did not resume after becoming readable.');
$resourceReader->close();
fclose($readerInput);

$writerStreams = stream_socket_pair(STREAM_PF_UNIX, STREAM_SOCK_STREAM, STREAM_IPPROTO_IP);
if ($writerStreams === false) {
    throw new RuntimeException('Unable to create writer test streams.');
}

[$writerOutput, $writerInput] = $writerStreams;
$resourceWriter = new ResourceWriter($writerOutput);
$resourceWriter->write('written');
expect(fread($writerInput, 7) === 'written', 'The resource writer did not write all bytes.');
$resourceWriter->close();
fclose($writerOutput);
fclose($writerInput);

$backpressureStreams = stream_socket_pair(STREAM_PF_UNIX, STREAM_SOCK_STREAM, STREAM_IPPROTO_IP);
if ($backpressureStreams === false) {
    throw new RuntimeException('Unable to create backpressure test streams.');
}

[$backpressureOutput, $backpressureInput] = $backpressureStreams;
stream_set_blocking($backpressureInput, false);
$backpressureWriter = new ResourceWriter($backpressureOutput);
$firstWrite = str_repeat('a', 2_097_152);
$secondWrite = str_repeat('b', 2_097_152);
$expectedWrite = $firstWrite . $secondWrite;
$receivedWrite = '';
$writesCompleted = EventLoop::getSuspension();
$readWatcher = EventLoop::onReadable($backpressureInput, static function () use (
    $backpressureInput,
    $expectedWrite,
    &$receivedWrite,
    $writesCompleted,
): void {
    while (($chunk = fread($backpressureInput, 65_536)) !== false && $chunk !== '') {
        $receivedWrite .= $chunk;
    }

    if (strlen($receivedWrite) === strlen($expectedWrite)) {
        $writesCompleted->resume();
    }
});
$writeTimeout = EventLoop::delay(2.0, static function () use ($writesCompleted): void {
    $writesCompleted->throw(new RuntimeException('Concurrent writes did not drain within two seconds.'));
});
EventLoop::queue(static function () use ($backpressureWriter, $firstWrite): void {
    $backpressureWriter->write($firstWrite);
});
EventLoop::queue(static function () use ($backpressureWriter, $secondWrite): void {
    $backpressureWriter->write($secondWrite);
});
$writesCompleted->suspend();
EventLoop::cancel($writeTimeout);
EventLoop::cancel($readWatcher);
expect($receivedWrite === $expectedWrite, 'Concurrent backpressured writes were interleaved.');
$backpressureWriter->close();
fclose($backpressureOutput);
fclose($backpressureInput);

$cancelled = false;
$cancellation = new SignalCancellationToken();
$subscription = $cancellation->subscribe(static function (CancelledException $_exception) use (&$cancelled): void {
    $cancelled = true;
});
$cancellation->cancel();
expect($subscription > 0 && $cancelled, 'Cancellation subscribers must be notified.');

$cancellationThrown = false;
try {
    $cancellation->throwIfCancelled();
} catch (CancelledException) {
    $cancellationThrown = true;
}

expect($cancellationThrown, 'Cancelled requests must throw from throwIfCancelled().');

$invalidSpanRejected = false;
try {
    new Span(2, 1);
} catch (InvalidArgumentException) {
    $invalidSpanRejected = true;
}

expect($invalidSpanRejected, 'Invalid spans must be rejected.');

echo "Mago SDK tests passed.\n";
