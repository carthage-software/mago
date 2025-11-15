<?php

class Template
{
    public function getName(): string
    {
        return 'template-name';
    }
}

class Configuration
{
    public function getTemplate(): Template
    {
        return new Template();
    }
}

class FormItem
{
    public function __construct(
        public readonly null|Configuration $configuration,
    ) {}
}

function getTemplateName(FormItem $item): null|string
{
    return $item->configuration?->getTemplate()->getName();
}

$item1 = new FormItem(new Configuration());
$item2 = new FormItem(null);

$result1 = getTemplateName($item1); // "template-name"
$result2 = getTemplateName($item2); // null
