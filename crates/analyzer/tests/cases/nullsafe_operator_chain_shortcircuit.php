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

/**
 * The nullsafe operator short-circuits the ENTIRE chain when the base is null.
 * Since getTemplate() returns non-nullable Template, the null in the result type
 * comes ONLY from the nullsafe short-circuit, not from getTemplate().
 * Therefore, PHP will never actually call ->getName() on null - it short-circuits.
 *
 * NO error should be reported here.
 */
function getTemplateName(FormItem $item): null|string
{
    return $item->configuration?->getTemplate()->getName();
}

$item1 = new FormItem(new Configuration());
$item2 = new FormItem(null);

$result1 = getTemplateName($item1); // "template-name"
$result2 = getTemplateName($item2); // null
