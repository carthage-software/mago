<?php

class Row
{
    private array $cells;

    public function __construct(array $cells)
    {
        $this->cells = $cells;
    }

    public function getCell(int $index): mixed
    {
        return $this->cells[$index] ?? null;
    }
}

interface SheetInterface
{
    public function getName(): string;

    /**
     * @return Iterator<Row>
     */
    public function getRowIterator(): Iterator;
}

class ExcelImporter
{
    /**
     */
    public function importSheet(SheetInterface $sheet, ?string $parent = null): array
    {
        $iter = $sheet->getRowIterator();
        $rows = iterator_to_array($iter);
        assert(count($rows) >= 1, 'Expected at least one row in the "' . $sheet->getName() . '" sheet');

        $data = [];

        if (count($rows) === 1 && $parent === null) {
            $rows[] = new Row([]);
        }

        for ($i = 2; $i <= count($rows); $i++) {
            $row = $rows[$i];

            $key = $row->getCell(1);
            $value = $row->getCell(2);

            if ($key === null || $value === null) {
                $message = 'Both key and value are required in row ' . $i . ' of the "' . $sheet->getName() . '" sheet';

                throw new Exception($message);
            }
        }

        return $data;
    }
}
