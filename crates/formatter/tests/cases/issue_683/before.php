<?php
class AClass
{
    public function run(): void
    {
        // When result and function are 15 long together, formatter will change every run
        $result = array_map(fn(array $row): string => (
            reset($row) ?: ''
        ), $this->function1('FOOO BARBAR FOOO barbarbar'));

        $result = array_map(
            fn(array $row): string => reset($row) ?: '',
            $this->function1('FOOO BARBAR FOOO barbarbar'),
        );
    }
}
