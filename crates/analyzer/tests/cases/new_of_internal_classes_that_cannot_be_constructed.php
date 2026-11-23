<?php

declare(strict_types=1);

function make(): void
{
    new Generator();
    new WeakReference();
    new XMLParser();
    new InflateContext();
    new DeflateContext();
    new Shmop();
    new GdImage();
    new PgSql\Connection();
    new LDAP\Connection();
    new FTP\Connection();
    new Dba\Connection();
}

/** @extends Generator<mixed, mixed, mixed, mixed> */
final class G extends Generator
{
}

function allowed(object $o): void
{
    WeakReference::create($o);
}
