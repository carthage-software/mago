<?php

declare(strict_types=1);

interface InhReader
{
    public function read(): string;
}

interface InhWriter
{
    public function write(string $data): void;
}

class InhReadWriter implements InhReader, InhWriter
{
    public function read(): string
    {
        return '';
    }

    public function write(string $data): void
    {
    }
}

$rw = new InhReadWriter();
$rw->write($rw->read());
