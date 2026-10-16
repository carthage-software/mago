<?php

class Item {
    public function isValid(): bool {
        return true;
    }
}

class Converter {
    public function convert(string $val): ?Item {
        return $val === '' ? null : new Item();
    }
}

$val = 'some string';
$c = new Converter();

if (($val = $c->convert($val)) === null || !$val->isValid()) {
}
