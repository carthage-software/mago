<?php

declare(strict_types=1);

function drain(Generator $generator): string
{
    try {
        $generator->next();
    } catch (ClosedGeneratorException $e) {
        return $e->getMessage();
    }

    return '';
}

function mangled(ReflectionProperty $property): string
{
    return $property->getMangledName();
}

function by_class(Dom\Element $element): int
{
    return $element->getElementsByClassName('item')->length;
}

function likely(string $locale): string|false
{
    return locale_add_likely_subtags($locale);
}

echo RegexIterator::INVERT_MATCH, SHUT_RDWR, T_PIPE, FILTER_THROW_ON_FAILURE;

function more(ReflectionConstant $constant, Dom\Document $document, Dom\Element $element, CurlSharePersistentHandle $share): void
{
    echo $constant->getFileName() ?: '', $constant->getExtensionName() ?: '', count($constant->getAttributes());
    echo $document->children->length, $element->children->length, $element->outerHTML, count($share->options);
    echo CURLFOLLOW_ALL, IMAGETYPE_SVG, FilesystemIterator::OTHER_MODE_MASK, ZipArchive::ER_TRUNCATED_ZIP;
    echo opcache_is_script_cached_in_file_cache(__FILE__) ? 'y' : 'n', locale_minimize_subtags('en') ?: '';
}

function filter_failure(): ?Filter\FilterFailedException
{
    return null;
}
