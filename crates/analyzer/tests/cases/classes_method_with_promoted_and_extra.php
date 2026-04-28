<?php

declare(strict_types=1);

final class ClassesPromotedAndExtra
{
    public string $note = '';

    public function __construct(public int $id, string $note)
    {
        $this->note = $note;
    }
}

echo (new ClassesPromotedAndExtra(1, 'n'))->note;
