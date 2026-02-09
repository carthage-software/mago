<?php

declare(strict_types=1);

$foo = new class {
    /** @var list<int> */
    public array $list = [];
};

$foo->list = [1, 2, 3, 4, 5, 6, 7];
array_splice(array: $foo->list, offset: 1, length: 2);

$foo = new class {
    /** @var non-empty-list<int> */
    public array $list = [1, 2];
};

array_splice(array: $foo->list, offset: 1, length: 2, replacement: [8, 9, 10]);
