<?php

function b()
{
    Psl\invariant(null === Iter\search(Vec\concat(...Vec\map($this->section->getRows(), static fn(ProductRow $row): array => $row->getFields())), static fn(ProductField $existing): bool => $existing->getName() === $field->getName()), 'Field names need to be unique per section.');
}
