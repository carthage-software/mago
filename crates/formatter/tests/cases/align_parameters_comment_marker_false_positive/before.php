<?php

final class ParameterAttributeMarkerStrings
{
    public function __construct(
        #[Example(pattern: '//')]
        private readonly FooBarBaz $first,
        #[Example(pattern: '/*')]
        private readonly string $second,
        #[Example(pattern: '#')]
        private readonly ?VeryLongTypeName $third = null,
    ) {}
}
