<?php

final readonly class DataProcessor
{
    /**
     * @param array{type: string} $data
     */
    public function processData(array $data): string
    {
        $value = $data['type'];

        return $value;
    }
}
