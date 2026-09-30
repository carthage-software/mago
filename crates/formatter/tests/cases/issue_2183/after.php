<?php

declare(strict_types=1);

namespace App\Models;

use DateInvalidTimeZoneException;
use DateTimeImmutable;
use DateTimeZone;

final class Test
{
    private const string TIMESTAMPS_TIMEZONE = 'Europe/Moscow';

    /**
     * @throws DateInvalidTimeZoneException
     * @api
     */
    public static function getTimeInTimeZone(): string
    {
        $object = (object) ['created_at' => new DateTimeImmutable('now')->format('c')];

        $createdAt = new DateTimeImmutable($object->created_at)->setTimezone(
            new DateTimeZone(self::TIMESTAMPS_TIMEZONE),
        );

        return $createdAt->format('c');
    }

    public function report(): void
    {
        $formattedReportTimestamp = formatTimestampForTheReport(
            self::DEFAULT_REPORT_TIMESTAMP_FORMAT_NAME_LONG_ENOUGH_AND_MORE,
        );
        $formattedReportTimestamp = $this->formatter->formatTimestampForTheReportOutput(
            Foo::DEFAULT_REPORT_TIMESTAMP_FORMAT_NAME_LONG,
        );

        $a = format(self::FORMAT);
        $b = $this->format(Foo::FORMAT);
    }
}
