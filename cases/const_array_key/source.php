<?php

class Example
{
    public const HOUR_TYPES = [
        'normal',
        'level1',
        'level2',
        'level3',
        'notWorkedHoliday',
    ];

    private array $hourThresholds;

    public function __construct(array $hourThresholds)
    {
        $this->hourThresholds = $hourThresholds;
    }

    /**
     * @return non-empty-list<mixed>
     */
    public function getThresholds(): array
    {
        $result = [];

        for ($i = 0; $i < 4; $i++) {
            $result[] = $this->hourThresholds[self::HOUR_TYPES[$i]] ?? null;
        }

        return $result;
    }
}
