<?php

use DateTime;

final readonly class DataProcessor
{
    /**
     * @param array{dateTime: string} $data
     */
    public function processData(array $data): string
    {
        $value = $data['dateTime'];

        return $value;
    }
}
