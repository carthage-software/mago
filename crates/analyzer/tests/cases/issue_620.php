<?php

declare(strict_types=1);

class Row
{
    public function getValue(): string
    {
        return 'value';
    }
}

class Rows
{
    /**
     * @param array<Row> $elements
     */
    public function __construct(
        private array $elements,
    ) {}

    public function first(): null|Row
    {
        return $this->elements[0] ?? null;
    }

    /**
     * @return array<Row>
     */
    public function all(): array
    {
        return $this->elements;
    }
}

interface Extractor
{
    /**
     * @return \Generator<Rows>
     */
    public function extract(): \Generator;
}

interface Pipeline
{
    /**
     * @return \Generator<Rows>
     */
    public function process(): \Generator;
}

class ExtractorImpl implements Extractor
{
    #[\Override]
    public function extract(): \Generator
    {
        yield new Rows([
            new Row(),
            new Row(),
        ]);

        yield new Rows([
            new Row(),
            new Row(),
        ]);
    }
}

class PipelineImpl implements Pipeline
{
    #[\Override]
    public function process(): \Generator
    {
        yield from (new ExtractorImpl())->extract();
    }
}

function process_row(Row $row): void
{
    echo $row->getValue() . "\n";
}

function process_rows(Rows $rows): void
{
    foreach ($rows->all() as $row) {
        process_row($row);
    }
}

$pipeline = new PipelineImpl();

foreach ($pipeline->process() as $rows) {
    process_rows($rows);
}

$result = $pipeline->process();
$result = \iterator_to_array($result);
foreach ($result as $rows) {
    process_rows($rows);
}
