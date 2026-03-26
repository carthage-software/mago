<?php declare(strict_types=1);

class MagoRocks
{
    public function __construct(
        public ?int $length,
        public string $start,
        public string $end,
    ) {}
}

/**
 * @throws Exception
 */
function test(MagoRocks $foo): int
{
    if ($foo->length !== null && $foo->length > 0) {
        $time1 = DateTimeImmutable::createFromFormat('Y-m-d*H:i:sT', $foo->start, new DateTimeZone('UTC'));
        $time2 = DateTimeImmutable::createFromFormat('Y-m-d*H:i:sT', $foo->end, new DateTimeZone('UTC'));
        if ($time1 !== false && $time2 !== false) {
            $diff = $time2->diff($time1, true);
            return $foo->length;
        }
    }

    throw new Exception('bar');
}
