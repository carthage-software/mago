<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Io;

use Mago\Sdk\Internal\Io\ResourceReader;
use Mago\Sdk\Internal\Io\ResourceWriter;
use PHPUnit\Framework\TestCase;
use Revolt\EventLoop;
use RuntimeException;

use function fclose;
use function fread;
use function fwrite;
use function str_repeat;
use function stream_set_blocking;
use function stream_socket_pair;
use function strlen;

use const PHP_OS_FAMILY;
use const STREAM_IPPROTO_IP;
use const STREAM_PF_INET;
use const STREAM_PF_UNIX;
use const STREAM_SOCK_STREAM;

final class ResourceTest extends TestCase
{
    public function testReaderWaitsForReadableStream(): void
    {
        [$input, $output] = self::createStreamPair();
        $reader = new ResourceReader($input);
        EventLoop::delay(0.001, static function () use ($output): void {
            fwrite($output, 'ready');
            fclose($output);
        });

        self::assertSame('ready', $reader->readExactly(5));
        $reader->close();
        fclose($input);
    }

    public function testWriterWritesAllBytes(): void
    {
        [$output, $input] = self::createStreamPair();
        $writer = new ResourceWriter($output);

        $writer->write('written');

        self::assertSame('written', fread($input, 7));
        $writer->close();
        fclose($output);
        fclose($input);
    }

    public function testConcurrentBackpressuredWritesRemainOrdered(): void
    {
        [$output, $input] = self::createStreamPair();
        stream_set_blocking($input, false);
        $writer = new ResourceWriter($output);
        $firstWrite = str_repeat('a', 2_097_152);
        $secondWrite = str_repeat('b', 2_097_152);
        $expectedWrite = $firstWrite . $secondWrite;
        $receivedWrite = '';
        $writesCompleted = EventLoop::getSuspension();
        $readWatcher = EventLoop::onReadable($input, static function () use (
            $input,
            $expectedWrite,
            &$receivedWrite,
            $writesCompleted,
        ): void {
            while (($chunk = fread($input, 65_536)) !== false && $chunk !== '') {
                $receivedWrite .= $chunk;
            }

            if (strlen($receivedWrite) === strlen($expectedWrite)) {
                $writesCompleted->resume();
            }
        });
        $writeTimeout = EventLoop::delay(2.0, static function () use ($writesCompleted): void {
            $writesCompleted->throw(new RuntimeException('Concurrent writes did not drain within two seconds.'));
        });
        EventLoop::queue(static function () use ($writer, $firstWrite): void {
            $writer->write($firstWrite);
        });
        EventLoop::queue(static function () use ($writer, $secondWrite): void {
            $writer->write($secondWrite);
        });

        $writesCompleted->suspend();
        EventLoop::cancel($writeTimeout);
        EventLoop::cancel($readWatcher);

        self::assertSame($expectedWrite, $receivedWrite);
        $writer->close();
        fclose($output);
        fclose($input);
    }

    /**
     * @return array{resource, resource}
     */
    private static function createStreamPair(): array
    {
        $domain = PHP_OS_FAMILY === 'Windows' ? STREAM_PF_INET : STREAM_PF_UNIX;
        $streams = stream_socket_pair($domain, STREAM_SOCK_STREAM, STREAM_IPPROTO_IP);
        if ($streams === false) {
            throw new RuntimeException('Unable to create SDK test streams.');
        }

        return $streams;
    }
}
