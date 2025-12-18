<?php

function start(null|string $id): void
{
    $id ?? throw new RuntimeException();
    process($id);
}

function process(string $_): void
{
}
