<?php

declare(strict_types=1);

/**
 * @phpstan-type Tag stdClass&object{name: string}
 * @phpstan-type Item stdClass&object{tags: list<Tag>}
 **/
class Foo
{
    /** @return Item */
    public static function make(): stdClass
    {
        /** @var Item $res */
        $res = new stdClass();
        $res->tags = [];

        return $res;
    }
}

$item = Foo::make();
$tags = $item->tags;
