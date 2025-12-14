<?php

declare(strict_types=1);

class Data
{
    public null|string $value = null;
    public null|int $count = null;
}

class Modifier
{
    public function reset(Data $data): void
    {
        $data->value = null;
        $data->count = null;
    }
}

function testDirectModification(): void
{
    $data = new Data();
    $data->value = 'hello';
    $data->count = 42;

    $modifier = new Modifier();
    $modifier->reset($data);

    if ($data->value === null) {
        echo 'value is null';
    }

    if ($data->count === null) {
        echo 'count is null';
    }
}

class Container
{
    public Data $data;

    public function __construct()
    {
        $this->data = new Data();
    }
}

class ContainerModifier
{
    public function reset(Container $container): void
    {
        $container->data->value = null;
        $container->data->count = null;
    }
}

function testNestedModification(): void
{
    $container = new Container();
    $container->data->value = 'world';
    $container->data->count = 100;

    $modifier = new ContainerModifier();
    $modifier->reset($container);

    if ($container->data->value === null) {
        echo 'nested value is null';
    }
}
