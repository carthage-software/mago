<?php

declare(strict_types=1);

class Template {}

/**
 * @throws LogicException
 */
function from_class_exists(string $templateClass): void
{
    if (!class_exists($templateClass) || !is_a($templateClass, Template::class, allow_string: true)) {
        throw new LogicException('not a template');
    }

    accepts_template_class($templateClass);
}

/**
 * @param class-string $templateClass
 *
 * @throws LogicException
 */
function from_class_string_param(string $templateClass): void
{
    if (!is_a($templateClass, Template::class, allow_string: true)) {
        throw new LogicException('not a template');
    }

    accepts_template_class($templateClass);
}

function accepts_object(object $value): void
{
    if (is_a($value, Template::class)) {
        accepts_template($value);
    }
}

/**
 * @param class-string<Template> $class
 */
function accepts_template_class(string $class): void
{
    echo $class;
}

function accepts_template(Template $template): void
{
    echo $template::class;
}
