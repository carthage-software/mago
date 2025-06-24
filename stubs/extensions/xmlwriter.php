<?php

function xmlwriter_open_uri(string $uri): XMLWriter|false
{
}

function xmlwriter_open_memory(): XMLWriter|false
{
}

function xmlwriter_set_indent(XMLWriter $writer, bool $enable): bool
{
}

function xmlwriter_set_indent_string(XMLWriter $writer, string $indentation): bool
{
}

function xmlwriter_start_comment(XMLWriter $writer): bool
{
}

function xmlwriter_end_comment(XMLWriter $writer): bool
{
}

function xmlwriter_start_attribute(XMLWriter $writer, string $name): bool
{
}

function xmlwriter_end_attribute(XMLWriter $writer): bool
{
}

function xmlwriter_write_attribute(XMLWriter $writer, string $name, string $value): bool
{
}

function xmlwriter_start_attribute_ns(
    XMLWriter $writer,
    null|string $prefix,
    string $name,
    null|string $namespace,
): bool {
}

function xmlwriter_write_attribute_ns(
    XMLWriter $writer,
    null|string $prefix,
    string $name,
    null|string $namespace,
    string $value,
): bool {
}

function xmlwriter_start_element(XMLWriter $writer, string $name): bool
{
}

function xmlwriter_end_element(XMLWriter $writer): bool
{
}

function xmlwriter_full_end_element(XMLWriter $writer): bool
{
}

function xmlwriter_start_element_ns(XMLWriter $writer, null|string $prefix, string $name, null|string $namespace): bool
{
}

function xmlwriter_write_element(XMLWriter $writer, string $name, null|string $content = null): bool
{
}

function xmlwriter_write_element_ns(
    XMLWriter $writer,
    null|string $prefix,
    string $name,
    null|string $namespace,
    null|string $content = null,
): bool {
}

function xmlwriter_start_pi(XMLWriter $writer, string $target): bool
{
}

function xmlwriter_end_pi(XMLWriter $writer): bool
{
}

function xmlwriter_write_pi(XMLWriter $writer, string $target, string $content): bool
{
}

function xmlwriter_start_cdata(XMLWriter $writer): bool
{
}

function xmlwriter_end_cdata(XMLWriter $writer): bool
{
}

function xmlwriter_write_cdata(XMLWriter $writer, string $content): bool
{
}

function xmlwriter_text(XMLWriter $writer, string $content): bool
{
}

function xmlwriter_write_raw(XMLWriter $writer, string $content): bool
{
}

function xmlwriter_start_document(
    XMLWriter $writer,
    null|string $version = '1.0',
    null|string $encoding = null,
    null|string $standalone = null,
): bool {
}

function xmlwriter_end_document(XMLWriter $writer): bool
{
}

function xmlwriter_write_comment(XMLWriter $writer, string $content): bool
{
}

function xmlwriter_start_dtd(
    XMLWriter $writer,
    string $qualifiedName,
    null|string $publicId = null,
    null|string $systemId = null,
): bool {
}

function xmlwriter_end_dtd(XMLWriter $writer): bool
{
}

function xmlwriter_write_dtd(
    XMLWriter $writer,
    string $name,
    null|string $publicId = null,
    null|string $systemId = null,
    null|string $content = null,
): bool {
}

function xmlwriter_start_dtd_element(XMLWriter $writer, string $qualifiedName): bool
{
}

function xmlwriter_end_dtd_element(XMLWriter $writer): bool
{
}

function xmlwriter_write_dtd_element(XMLWriter $writer, string $name, string $content): bool
{
}

function xmlwriter_start_dtd_attlist(XMLWriter $writer, string $name): bool
{
}

function xmlwriter_end_dtd_attlist(XMLWriter $writer): bool
{
}

function xmlwriter_write_dtd_attlist(XMLWriter $writer, string $name, string $content): bool
{
}

function xmlwriter_start_dtd_entity(XMLWriter $writer, string $name, bool $isParam): bool
{
}

function xmlwriter_end_dtd_entity(XMLWriter $writer): bool
{
}

function xmlwriter_write_dtd_entity(
    XMLWriter $writer,
    string $name,
    string $content,
    bool $isParam = false,
    null|string $publicId = null,
    null|string $systemId = null,
    null|string $notationData = null,
): bool {
}

function xmlwriter_output_memory(XMLWriter $writer, bool $flush = true): string
{
}

function xmlwriter_flush(XMLWriter $writer, bool $empty = true): string|int
{
}

class XMLWriter
{
    public function openUri(string $uri): bool
    {
    }

    public static function toUri(string $uri): static
    {
    }

    public function openMemory(): bool
    {
    }

    public static function toMemory(): static
    {
    }

    /** @param resource $stream */
    public static function toStream($stream): static
    {
    }

    public function setIndent(bool $enable): bool
    {
    }

    public function setIndentString(string $indentation): bool
    {
    }

    public function startComment(): bool
    {
    }

    public function endComment(): bool
    {
    }

    public function startAttribute(string $name): bool
    {
    }

    public function endAttribute(): bool
    {
    }

    public function writeAttribute(string $name, string $value): bool
    {
    }

    public function startAttributeNs(null|string $prefix, string $name, null|string $namespace): bool
    {
    }

    public function writeAttributeNs(null|string $prefix, string $name, null|string $namespace, string $value): bool
    {
    }

    public function startElement(string $name): bool
    {
    }

    public function endElement(): bool
    {
    }

    public function fullEndElement(): bool
    {
    }

    public function startElementNs(null|string $prefix, string $name, null|string $namespace): bool
    {
    }

    public function writeElement(string $name, null|string $content = null): bool
    {
    }

    public function writeElementNs(
        null|string $prefix,
        string $name,
        null|string $namespace,
        null|string $content = null,
    ): bool {
    }

    public function startPi(string $target): bool
    {
    }

    public function endPi(): bool
    {
    }

    public function writePi(string $target, string $content): bool
    {
    }

    public function startCdata(): bool
    {
    }

    public function endCdata(): bool
    {
    }

    public function writeCdata(string $content): bool
    {
    }

    public function text(string $content): bool
    {
    }

    public function writeRaw(string $content): bool
    {
    }

    public function startDocument(
        null|string $version = '1.0',
        null|string $encoding = null,
        null|string $standalone = null,
    ): bool {
    }

    public function endDocument(): bool
    {
    }

    public function writeComment(string $content): bool
    {
    }

    public function startDtd(string $qualifiedName, null|string $publicId = null, null|string $systemId = null): bool
    {
    }

    public function endDtd(): bool
    {
    }

    public function writeDtd(
        string $name,
        null|string $publicId = null,
        null|string $systemId = null,
        null|string $content = null,
    ): bool {
    }

    public function startDtdElement(string $qualifiedName): bool
    {
    }

    public function endDtdElement(): bool
    {
    }

    public function writeDtdElement(string $name, string $content): bool
    {
    }

    public function startDtdAttlist(string $name): bool
    {
    }

    public function endDtdAttlist(): bool
    {
    }

    public function writeDtdAttlist(string $name, string $content): bool
    {
    }

    public function startDtdEntity(string $name, bool $isParam): bool
    {
    }

    public function endDtdEntity(): bool
    {
    }

    public function writeDtdEntity(
        string $name,
        string $content,
        bool $isParam = false,
        null|string $publicId = null,
        null|string $systemId = null,
        null|string $notationData = null,
    ): bool {
    }

    public function outputMemory(bool $flush = true): string
    {
    }

    public function flush(bool $empty = true): string|int
    {
    }
}
