<?php

declare(strict_types=1);

namespace Mago\Tests\Unit;

use PHPUnit\Framework\TestCase;

use function Mago\Internal\format_progress;
use function Mago\Internal\progress_due;

use const Mago\Internal\PROGRESS_INTERVAL_NS;

final class DownloadProgressTest extends TestCase
{
    public function testHalfMebibyteSampleIsFormatted(): void
    {
        self::assertSame("\r  0.5 / 1.0 MB (50%)", format_progress(['total' => 1_048_576, 'downloaded' => 524_288]));
    }

    public function testMultiMebibyteSampleIsFormatted(): void
    {
        self::assertSame("\r  2.9 / 7.6 MB (37%)", format_progress(['total' => 8_000_000, 'downloaded' => 3_000_000]));
    }

    public function testFirstSampleIsDue(): void
    {
        self::assertTrue(progress_due(null, 42));
    }

    public function testSampleInsideTheIntervalIsNotDue(): void
    {
        self::assertFalse(progress_due(1_000, 1_000 + PROGRESS_INTERVAL_NS - 1));
    }

    public function testSampleAtTheIntervalIsDue(): void
    {
        self::assertTrue(progress_due(1_000, 1_000 + PROGRESS_INTERVAL_NS));
    }
}
