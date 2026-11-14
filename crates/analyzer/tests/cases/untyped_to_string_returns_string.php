<?php

declare(strict_types=1);

final class Name
{
    public function __toString()
    {
        return 'name';
    }
}

interface Labelled
{
    public function __toString();
}

trait Printable
{
    public function __toString()
    {
        return 'printable';
    }
}

final class UsesTrait
{
    use Printable;
}

function from_class(Name $name): string
{
    return $name->__toString();
}

function from_interface(Labelled $labelled): string
{
    return $labelled->__toString();
}

function from_trait(UsesTrait $value): string
{
    return $value->__toString();
}
